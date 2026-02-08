import argparse
import json
import time
import uuid
import urllib.request
import urllib.error
from concurrent.futures import ThreadPoolExecutor
from typing import Any


def http_post_json(
	base_url: str,
	path: str,
	payload: dict[str, Any],
	timeout_s: float,
) -> int:
	data = json.dumps(payload).encode("utf-8")
	req = urllib.request.Request(
		base_url + path,
		data=data,
		headers={"Content-Type": "application/json"},
		method="POST",
	)
	with urllib.request.urlopen(req, timeout=timeout_s) as resp:
		resp.read()
		return resp.status


def http_get_json(base_url: str, path: str, timeout_s: float) -> Any | None:
	req = urllib.request.Request(
		base_url + path,
		headers={"Accept": "application/json"},
		method="GET",
	)
	with urllib.request.urlopen(req, timeout=timeout_s) as resp:
		body = resp.read().decode("utf-8")
		if not body:
			return None
		return json.loads(body)


def get_metrics(base_url: str, timeout_s: float) -> dict[str, Any] | None:
	try:
		return http_get_json(base_url, "/metrics", timeout_s)
	except Exception:
		return None


def run_client(
	base_url: str,
	user_id: str,
	per_client: int,
	amount: float,
	timeout_s: float,
) -> tuple[int, int]:
	ok = 0
	fail = 0
	payload = {"user_id": user_id, "amount": amount}

	for _ in range(per_client):
		try:
			status = http_post_json(base_url, "/transaction", payload, timeout_s)
			if 200 <= status < 300:
				ok += 1
			else:
				fail += 1
		except Exception:
			fail += 1

	return ok, fail


def diff_metrics(
	before: dict[str, Any] | None,
	after: dict[str, Any] | None,
) -> dict[str, int] | None:
	if not before or not after:
		return None

	return {
		"counter_time": max(0, after.get("counter_time", 0) - before.get("counter_time", 0)),
		"logging_time": max(0, after.get("logging_time", 0) - before.get("logging_time", 0)),
	}


def verify_distinct_accounts(
	base_url: str,
	user_ids: list[str],
	expected_balance: float,
	timeout_s: float,
) -> tuple[bool, str]:
	try:
		balances = http_get_json(base_url, "/accounts", timeout_s)
	except Exception:
		return False, "failed to fetch /accounts"

	if not isinstance(balances, dict):
		return False, "unexpected /accounts response"

	for user_id in user_ids:
		balance = balances.get(user_id)
		if balance is None:
			return False, f"missing account {user_id}"
		if abs(balance - expected_balance) > 1e-9:
			return False, f"account {user_id} balance {balance} != {expected_balance}"

	return True, "ok"


def verify_single_account(
	base_url: str,
	user_id: str,
	expected_balance: float,
	timeout_s: float,
) -> tuple[bool, str]:
	try:
		payload = http_get_json(base_url, f"/user/{user_id}", timeout_s)
	except Exception:
		return False, "failed to fetch /user/{user_id}"

	if not isinstance(payload, dict):
		return False, "unexpected /user response"

	balance = payload.get("balance")
	if balance is None:
		return False, "missing balance in /user response"
	if abs(balance - expected_balance) > 1e-9:
		return False, f"balance {balance} != {expected_balance}"

	return True, "ok"


def verify_users_via_user_endpoint(
	base_url: str,
	user_ids: list[str],
	expected_balance: float,
	timeout_s: float,
) -> tuple[bool, str]:
	for user_id in user_ids:
		ok, msg = verify_single_account(
			base_url,
			user_id,
			expected_balance,
			timeout_s,
		)
		if not ok:
			return False, msg

	return True, "ok"


def run_scenario(
	name: str,
	base_url: str,
	user_ids: list[str],
	per_client: int,
	amount: float,
	timeout_s: float,
) -> tuple[float, int, int]:
	total_requests = len(user_ids) * per_client
	metrics_before = get_metrics(base_url, timeout_s)

	start = time.perf_counter()
	ok_total = 0
	fail_total = 0

	with ThreadPoolExecutor(max_workers=len(user_ids)) as executor:
		futures = [
			executor.submit(run_client, base_url, user_id, per_client, amount, timeout_s)
			for user_id in user_ids
		]
		for future in futures:
			ok, fail = future.result()
			ok_total += ok
			fail_total += fail

	total_time = time.perf_counter() - start
	metrics_after = get_metrics(base_url, timeout_s)
	metrics_delta = diff_metrics(metrics_before, metrics_after)

	qps = total_requests / total_time if total_time > 0 else 0

	print("=" * 60)
	print(f"Scenario: {name}")
	print(f"Requests: {total_requests} (ok={ok_total}, fail={fail_total})")
	print(f"Total time: {total_time}")
	print(f"Throughput: {qps:.2f} req/s")

	if metrics_delta is None:
		print("Metrics: unavailable (failed to read /metrics)")
	else:
		counter_s = metrics_delta["counter_time"] / 1_000_000_000
		logging_s = metrics_delta["logging_time"] / 1_000_000_000
		print(
			"Metrics (delta): counter="
			f"{counter_s} s, logging={logging_s} s"
		)

	return total_time, ok_total, fail_total


def main() -> None:
	parser = argparse.ArgumentParser(description="Facade load test")
	parser.add_argument("--base-url", default="http://localhost:8080")
	parser.add_argument("--clients", type=int, default=10)
	parser.add_argument("--per-client", type=int, default=10_000)
	parser.add_argument("--amount", type=float, default=1.0)
	parser.add_argument("--timeout", type=float, default=10.0)
	parser.add_argument(
		"--scenario",
		choices=["distinct", "same", "both"],
		default="both",
	)

	args = parser.parse_args()

	if args.clients <= 0 or args.per_client <= 0:
		raise SystemExit("clients and per-client must be > 0")

	run_distinct = args.scenario in ("distinct", "both")
	run_same = args.scenario in ("same", "both")

	if run_distinct:
		user_ids = [
			f"distinct-{i}-{uuid.uuid4().hex[:8]}" for i in range(args.clients)
		]
		run_scenario(
			f"{args.clients} clients x {args.per_client} to distinct accounts",
			args.base_url,
			user_ids,
			args.per_client,
			args.amount,
			args.timeout,
		)
		ok, msg = verify_distinct_accounts(
			args.base_url, user_ids, args.per_client * args.amount, args.timeout
		)
		status = "OK" if ok else "FAIL"
		print(f"Verification: {status} ({msg})")
		ok, msg = verify_users_via_user_endpoint(
			args.base_url, user_ids, args.per_client * args.amount, args.timeout
		)
		status = "OK" if ok else "FAIL"
		print(f"Verification (/user): {status} ({msg})")

	if run_same:
		shared_user_id = f"shared-{uuid.uuid4().hex[:8]}"
		user_ids = [shared_user_id for _ in range(args.clients)]
		run_scenario(
			f"{args.clients} clients x {args.per_client} to same account",
			args.base_url,
			user_ids,
			args.per_client,
			args.amount,
			args.timeout,
		)
		expected = args.clients * args.per_client * args.amount
		ok, msg = verify_single_account(
			args.base_url, shared_user_id, expected, args.timeout
		)
		status = "OK" if ok else "FAIL"
		print(f"Verification: {status} ({msg})")


if __name__ == "__main__":
	main()

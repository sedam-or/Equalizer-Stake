"""Performance & Load Test — asyncio + httpx concurrent load testing.

Targets (NFR-PERF):
- PERF-01: Page load ≤2s (P95)
- PERF-02: Form validation ≤1s
- PERF-04: ≤50 concurrent users

Usage:
    cd /home/kepolu/bmdmusic/backend
    python -m tests.load_test
"""

import asyncio
import time
import statistics
from dataclasses import dataclass, field

import httpx

BASE = "http://localhost:8001"
CONCURRENT = 50


@dataclass
class TestResult:
    name: str
    durations: list[float] = field(default_factory=list)
    errors: int = 0
    status_codes: dict = field(default_factory=dict)

    @property
    def count(self): return len(self.durations) + self.errors

    @property
    def avg(self): return statistics.mean(self.durations) if self.durations else 0

    @property
    def p50(self): return statistics.median(self.durations) if self.durations else 0

    @property
    def p95(self):
        if not self.durations: return 0
        sorted_d = sorted(self.durations)
        idx = int(len(sorted_d) * 0.95)
        return sorted_d[min(idx, len(sorted_d) - 1)]

    @property
    def p99(self):
        if not self.durations: return 0
        sorted_d = sorted(self.durations)
        idx = int(len(sorted_d) * 0.99)
        return sorted_d[min(idx, len(sorted_d) - 1)]

    def summary(self):
        return (
            f"{self.name}: {self.count} req, {self.errors} errors, "
            f"avg={self.avg:.3f}s, p50={self.p50:.3f}s, p95={self.p95:.3f}s, "
            f"codes={self.status_codes}"
        )


async def timed_request(client: httpx.AsyncClient, method: str, url: str, **kwargs) -> tuple[float, int]:
    """Make request, return (duration_seconds, status_code)."""
    start = time.monotonic()
    try:
        r = await client.request(method, url, **kwargs)
        duration = time.monotonic() - start
        return (duration, r.status_code)
    except Exception:
        duration = time.monotonic() - start
        return (duration, 0)


async def load_test_health(client: httpx.AsyncClient) -> TestResult:
    """Load test: GET /health × CONCURRENT."""
    result = TestResult("GET /health")
    tasks = [timed_request(client, "GET", f"{BASE}/health") for _ in range(CONCURRENT)]
    responses = await asyncio.gather(*tasks)
    for dur, status in responses:
        if status == 0:
            result.errors += 1
        else:
            result.durations.append(dur)
            result.status_codes[status] = result.status_codes.get(status, 0) + 1
    return result


async def load_test_login(client: httpx.AsyncClient, username: str, password: str) -> TestResult:
    """Load test: POST /api/auth/login × CONCURRENT."""
    result = TestResult("POST /api/auth/login")
    payload = {"username": username, "password": password}
    tasks = [
        timed_request(client, "POST", f"{BASE}/api/auth/login", json=payload)
        for _ in range(CONCURRENT)
    ]
    responses = await asyncio.gather(*tasks)
    for dur, status in responses:
        if status == 0:
            result.errors += 1
        else:
            result.durations.append(dur)
            result.status_codes[status] = result.status_codes.get(status, 0) + 1
    return result


async def load_test_pricing(client: httpx.AsyncClient, token: str) -> TestResult:
    """Load test: GET /api/pricing/ × CONCURRENT."""
    result = TestResult("GET /api/pricing/")
    headers = {"Authorization": f"Bearer {token}"}
    tasks = [
        timed_request(client, "GET", f"{BASE}/api/pricing/", headers=headers)
        for _ in range(CONCURRENT)
    ]
    responses = await asyncio.gather(*tasks)
    for dur, status in responses:
        if status == 0:
            result.errors += 1
        else:
            result.durations.append(dur)
            result.status_codes[status] = result.status_codes.get(status, 0) + 1
    return result


async def load_test_settings(client: httpx.AsyncClient, token: str) -> TestResult:
    """Load test: GET /api/settings/ × CONCURRENT."""
    result = TestResult("GET /api/settings/")
    headers = {"Authorization": f"Bearer {token}"}
    tasks = [
        timed_request(client, "GET", f"{BASE}/api/settings/", headers=headers)
        for _ in range(CONCURRENT)
    ]
    responses = await asyncio.gather(*tasks)
    for dur, status in responses:
        if status == 0:
            result.errors += 1
        else:
            result.durations.append(dur)
            result.status_codes[status] = result.status_codes.get(status, 0) + 1
    return result


async def load_test_mixed(client: httpx.AsyncClient, token: str) -> TestResult:
    """Load test: mixed GET/POST × CONCURRENT."""
    result = TestResult("MIXED endpoints")
    headers = {"Authorization": f"Bearer {token}"}
    endpoints = [
        ("GET", f"{BASE}/health", {}),
        ("GET", f"{BASE}/api/pricing/", {"headers": headers}),
        ("GET", f"{BASE}/api/settings/", {"headers": headers}),
        ("GET", f"{BASE}/api/audit/", {"headers": headers}),
    ]
    tasks = []
    for _ in range(CONCURRENT):
        method, url, kwargs = endpoints[_ % len(endpoints)]
        tasks.append(timed_request(client, method, url, **kwargs))
    responses = await asyncio.gather(*tasks)
    for dur, status in responses:
        if status == 0:
            result.errors += 1
        else:
            result.durations.append(dur)
            result.status_codes[status] = result.status_codes.get(status, 0) + 1
    return result


def check_nfr(name: str, p95: float, threshold: float) -> bool:
    """Check if P95 meets NFR threshold."""
    passed = p95 <= threshold
    status = "✅" if passed else "❌"
    print(f"  {status} {name}: p95={p95:.3f}s (threshold: ≤{threshold}s)")
    return passed


async def main():
    print(f"{'='*60}")
    print(f"PERFORMANCE & LOAD TEST — {CONCURRENT} concurrent users")
    print(f"{'='*60}\n")

    # Get auth token
    async with httpx.AsyncClient(timeout=60.0) as client:
        r = await client.post(f"{BASE}/api/auth/login", json={"username": "owner_test", "password": "owner123"})
        if r.status_code != 200:
            print(f"ERROR: Cannot login. Status: {r.status_code}")
            return
        token = r.json().get("access_token") or r.json().get("token")

        # Run load tests
        print("Running load tests...")
        tests = [
            await load_test_health(client),
            await load_test_login(client, "owner_test", "owner123"),
            await load_test_pricing(client, token),
            await load_test_settings(client, token),
            await load_test_mixed(client, token),
        ]

    # Results
    print(f"\n{'='*60}")
    print("RESULTS")
    print(f"{'='*60}")
    all_pass = True
    for t in tests:
        print(f"\n{t.summary()}")

    # NFR checks
    print(f"\n{'='*60}")
    print("NFR COMPLIANCE CHECK")
    print(f"{'='*60}")
    all_pass &= check_nfr("NFR-PERF-01: Page load", tests[0].p95, 2.0)
    all_pass &= check_nfr("NFR-PERF-02: Form validation (login)", tests[1].p95, 1.0)
    all_pass &= check_nfr("NFR-PERF-04: Concurrent users (mixed)", tests[4].p95, 2.0)

    # Summary
    total_errors = sum(t.errors for t in tests)
    print(f"\n{'='*60}")
    print(f"SUMMARY: {len(tests)} tests, {sum(t.count for t in tests)} total requests, {total_errors} errors")
    if all_pass:
        print("ALL NFRs PASSED ✅")
    else:
        print("SOME NFRs FAILED ❌")


if __name__ == "__main__":
    asyncio.run(main())

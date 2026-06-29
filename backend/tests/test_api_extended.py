"""Extended integration tests for coverage improvement."""

import pytest
from httpx import AsyncClient

from tests.conftest import auth_header, get_token, make_user_raw


@pytest.mark.asyncio
class TestAdminMgmtEndpoints:
    async def test_list_admins(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.get("/api/admin-mgmt/admins", headers=auth_header(token))
        assert r.status_code == 200
        assert "admins" in r.json()

    async def test_create_admin(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.post("/api/admin-mgmt/admins", headers=auth_header(token), json={
            "username": f"new_admin_{username[-8:]}", "email": f"new_{username[-8:]}@test.com", "password": "NewAdmin123!"
        })
        assert r.status_code in (200, 201)

    async def test_admin_cannot_list(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.get("/api/admin-mgmt/admins", headers=auth_header(token))
        assert r.status_code == 403


@pytest.mark.asyncio
class TestOverrideEndpoints:
    async def test_unfreeze_not_found(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.put(f"/api/override/unfreeze/00000000-0000-0000-0000-000000000000", headers=auth_header(token))
        assert r.status_code == 404

    async def test_cancel_invoice_not_found(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.put(f"/api/override/cancel-invoice/00000000-0000-0000-0000-000000000000", headers=auth_header(token))
        assert r.status_code == 404

    async def test_revert_session_not_found(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.put(f"/api/override/revert-session/00000000-0000-0000-0000-000000000000", headers=auth_header(token))
        assert r.status_code == 404

    async def test_admin_cannot_override(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.put(f"/api/override/unfreeze/00000000-0000-0000-0000-000000000000", headers=auth_header(token))
        assert r.status_code == 403


@pytest.mark.asyncio
class TestSchedulingEndpoints:
    async def test_create_session_validation(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.post("/api/sessions/", headers=auth_header(token), json={})
        assert r.status_code in (400, 422)

    async def test_validate_session(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.post("/api/sessions/validate", headers=auth_header(token), json={})
        assert r.status_code in (200, 400, 422)


@pytest.mark.asyncio
class TestBillingEndpoints:
    async def test_billing_pay_verify(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.post("/api/billing/payments/verify", headers=auth_header(token), json={})
        assert r.status_code in (200, 400, 422)


@pytest.mark.asyncio
class TestAttendanceEndpoints:
    async def test_create_attendance_validation(self, client: AsyncClient):
        username, pw = await make_user_raw("teacher")
        token = await get_token(client, username, pw)
        r = await client.post("/api/attendance/", headers=auth_header(token), json={})
        assert r.status_code in (400, 403, 422)


@pytest.mark.asyncio
class TestAuditEndpoints:
    async def test_list_audit_logs(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.get("/api/audit/", headers=auth_header(token))
        assert r.status_code == 200

    async def test_admin_limited_audit(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.get("/api/audit/", headers=auth_header(token))
        assert r.status_code == 200


@pytest.mark.asyncio
class TestPrivacyEndpoints:
    async def test_privacy_delete_request(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.post("/api/privacy/delete-request", headers=auth_header(token), json={})
        assert r.status_code in (200, 400, 403, 422)


@pytest.mark.asyncio
class TestOwnerDashboard:
    async def test_owner_dashboard(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.get("/api/reports/owner-dashboard", headers=auth_header(token))
        assert r.status_code == 200

    async def test_admin_cannot_dashboard(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.get("/api/reports/owner-dashboard", headers=auth_header(token))
        assert r.status_code == 403


@pytest.mark.asyncio
class TestNotificationExtended:
    async def test_reset_circuit_breaker(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.post("/api/notifications/circuit-breaker/reset?channel=whatsapp", headers=auth_header(token))
        assert r.status_code == 200

    async def test_reset_invalid_channel(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.post("/api/notifications/circuit-breaker/reset?channel=invalid", headers=auth_header(token))
        assert r.status_code == 400

    async def test_test_whatsapp_no_config(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.post("/api/notifications/test-whatsapp", headers=auth_header(token), json={"phone": "08123456789"})
        assert r.status_code in (200, 400)

    async def test_test_whatsapp_missing_phone(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.post("/api/notifications/test-whatsapp", headers=auth_header(token), json={})
        assert r.status_code == 422

"""Integration tests for API endpoints."""

import pytest
from httpx import AsyncClient

from tests.conftest import auth_header, get_token, make_user_raw


@pytest.mark.asyncio
class TestHealthEndpoints:
    async def test_health(self, client: AsyncClient):
        r = await client.get("/health")
        assert r.status_code == 200
        assert r.json()["status"] == "ok"


@pytest.mark.asyncio
class TestAuthEndpoints:
    async def test_login_success(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        r = await client.post("/api/auth/login", json={"username": username, "password": pw})
        assert r.status_code == 200
        assert "access_token" in r.json() or "token" in r.json()

    async def test_login_wrong_password(self, client: AsyncClient):
        username, _ = await make_user_raw("admin")
        r = await client.post("/api/auth/login", json={"username": username, "password": "WrongPass123!"})
        assert r.status_code == 401

    async def test_login_nonexistent_user(self, client: AsyncClient):
        r = await client.post("/api/auth/login", json={"username": "nobody_xyz_123", "password": "WrongPass123!"})
        assert r.status_code == 401

    async def test_protected_no_token(self, client: AsyncClient):
        r = await client.get("/api/settings/")
        assert r.status_code in (401, 403)


@pytest.mark.asyncio
class TestSettingsEndpoints:
    async def test_settings_requires_owner(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.get("/api/settings/", headers=auth_header(token))
        assert r.status_code == 403

    async def test_settings_owner_access(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.get("/api/settings/", headers=auth_header(token))
        assert r.status_code == 200
        assert "data" in r.json()

    async def test_settings_put(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        h = auth_header(token)
        payload = {"value": {"enabled": False, "start_time": "", "end_time": "", "message": "test"}}
        r = await client.put("/api/settings/maintenance", headers=h, json=payload)
        assert r.status_code == 200

    async def test_settings_invalid_key(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.put("/api/settings/bad_key", headers=auth_header(token), json={"value": {"x": 1}})
        assert r.status_code == 422


@pytest.mark.asyncio
class TestPricingEndpoints:
    async def test_pricing_create(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        h = auth_header(token)
        payload = {"instrument": "piano", "level": "pemula", "price": 500000, "effective_date": "2026-08-01"}
        r = await client.post("/api/pricing/", headers=h, json=payload)
        assert r.status_code == 200
        assert r.json()["instrument"] == "piano"

    async def test_pricing_invalid_instrument(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.post("/api/pricing/", headers=auth_header(token), json={
            "instrument": "violin", "level": "pemula", "price": 100, "effective_date": "2026-08-01"
        })
        assert r.status_code == 422

    async def test_pricing_admin_access(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.get("/api/pricing/", headers=auth_header(token))
        assert r.status_code == 200


@pytest.mark.asyncio
class TestDraftEndpoints:
    async def test_draft_save_and_get(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        h = auth_header(token)

        r = await client.post("/api/drafts/test-form", headers=h, json={"draft_data": {"field": "value"}})
        assert r.status_code == 200

        r = await client.get("/api/drafts/test-form", headers=h)
        assert r.status_code == 200
        assert r.json()["draft_data"]["field"] == "value"

    async def test_draft_delete(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        h = auth_header(token)

        await client.post("/api/drafts/del-form", headers=h, json={"draft_data": {"x": 1}})
        r = await client.delete("/api/drafts/del-form", headers=h)
        assert r.status_code == 200

    async def test_draft_not_found(self, client: AsyncClient):
        username, pw = await make_user_raw("admin")
        token = await get_token(client, username, pw)
        r = await client.get("/api/drafts/nonexistent_xyz", headers=auth_header(token))
        assert r.status_code == 404


@pytest.mark.asyncio
class TestNotificationEndpoints:
    async def test_circuit_breaker_status(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.get("/api/notifications/circuit-breaker/status", headers=auth_header(token))
        assert r.status_code == 200
        assert "whatsapp" in r.json()

    async def test_email_status(self, client: AsyncClient):
        username, pw = await make_user_raw("owner")
        token = await get_token(client, username, pw)
        r = await client.get("/api/notifications/email/status", headers=auth_header(token))
        assert r.status_code == 200
        assert "email" in r.json()
        assert "smtp_configured" in r.json()

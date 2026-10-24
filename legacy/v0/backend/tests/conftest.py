"""Pytest configuration and fixtures for Equalizer-Stake backend tests."""

import asyncio
import uuid

import bcrypt
import pytest
import pytest_asyncio
from httpx import ASGITransport, AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.pool import NullPool

from app.core.database import get_db
from app.main import app

TEST_DSN = "postgresql+asyncpg://eqs_admin:eqs_dev_password@localhost:5432/equalizer_stake"
SYNC_DSN = "postgresql://eqs_admin:eqs_dev_password@localhost:5432/equalizer_stake"

engine = create_async_engine(TEST_DSN, echo=False, poolclass=NullPool)
TestSession = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)


@pytest.fixture(scope="session")
def event_loop():
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


@pytest_asyncio.fixture
async def client():
    """Provide test HTTP client with fresh DB session per request."""
    async def override_get_db():
        async with TestSession() as session:
            yield session

    app.dependency_overrides[get_db] = override_get_db
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as c:
        yield c
    app.dependency_overrides.clear()


def make_user_raw_sync(role: str = "admin") -> tuple[str, str]:
    """Create user via sync psycopg2, return (username, password)."""
    import psycopg2
    password = "Test1234!"
    pw_hash = bcrypt.hashpw(password.encode(), bcrypt.gensalt()).decode()
    username = f"test_{role}_{uuid.uuid4().hex[:8]}"
    email = f"test_{uuid.uuid4().hex[:8]}@test.com"
    uid = uuid.uuid4()

    conn = psycopg2.connect(SYNC_DSN)
    try:
        cur = conn.cursor()
        cur.execute(
            "INSERT INTO users (id_user, username, email, password_hash, role, is_active) "
            "VALUES (%s, %s, %s, %s, %s, true)",
            (str(uid), username, email, pw_hash, role),
        )
        conn.commit()
        cur.close()
    finally:
        conn.close()

    return username, password


async def make_user_raw(role: str = "admin") -> tuple[str, str]:
    """Create user via sync psycopg2 (async wrapper)."""
    return make_user_raw_sync(role)


async def get_token(client, username: str, password: str) -> str:
    """Login and return JWT token."""
    r = await client.post("/api/auth/login", json={"username": username, "password": password})
    assert r.status_code == 200, f"Login failed for {username}: {r.status_code} {r.text}"
    return r.json().get("access_token") or r.json().get("token")


def auth_header(token: str) -> dict:
    return {"Authorization": f"Bearer {token}"}

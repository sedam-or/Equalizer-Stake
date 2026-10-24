"""Create test user for rate limit testing."""
import bcrypt, asyncio, uuid
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy import text

pw = bcrypt.hashpw(b"TestPass123!", bcrypt.gensalt()).decode()
uid = str(uuid.uuid4())

async def main():
    engine = create_async_engine("postgresql+asyncpg://eqs_admin:eqs_dev_password@localhost:5433/equalizer_stake")
    async with engine.begin() as conn:
        await conn.execute(text("DELETE FROM users WHERE username='test_rl'"))
        await conn.execute(
            text("INSERT INTO users (id_user, username, email, password_hash, role, is_active, failed_login_attempts, first_login, is_temp_password) VALUES (:id, 'test_rl', 'rl@test.com', :pw, 'admin', true, 0, false, false)"),
            {"id": uid, "pw": pw}
        )
        print(f"Created test_rl id={uid}")

asyncio.run(main())

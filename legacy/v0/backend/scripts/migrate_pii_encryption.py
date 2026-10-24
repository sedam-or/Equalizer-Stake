"""T-090: Encrypt existing PII + add blind index columns.

Run: python -m scripts.migrate_pii_encryption
"""
import asyncio
import bcrypt
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy import text

from app.services.encryption import encrypt_pii, hash_pii


async def migrate():
    engine = create_async_engine(
        "postgresql+asyncpg://eqs_admin:eqs_dev_password@localhost:5433/equalizer_stake"
    )

    async with engine.begin() as conn:
        # 1. Add new columns + alter existing types (idempotent)
        await conn.execute(text("ALTER TABLE users ADD COLUMN IF NOT EXISTS email_hash VARCHAR(64)"))
        await conn.execute(text("ALTER TABLE guardians ADD COLUMN IF NOT EXISTS phone_hash VARCHAR(64)"))
        await conn.execute(text("ALTER TABLE guardians ALTER COLUMN phone_whatsapp TYPE VARCHAR(500)"))
        await conn.execute(text("ALTER TABLE guardians ALTER COLUMN email TYPE VARCHAR(500)"))
        await conn.execute(text("ALTER TABLE users ALTER COLUMN email TYPE VARCHAR(500)"))
        await conn.execute(text("ALTER TABLE registrations ALTER COLUMN phone_whatsapp TYPE VARCHAR(500)"))
        await conn.execute(text("ALTER TABLE registrations ALTER COLUMN email TYPE VARCHAR(500)"))
        print("Columns added/altered")

        # 2. Add unique indexes for blind index columns
        await conn.execute(text("CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_hash ON users(email_hash)"))
        await conn.execute(text("CREATE UNIQUE INDEX IF NOT EXISTS idx_guardians_phone_hash ON guardians(phone_hash)"))
        print("Indexes created (or already exist)")

        # 3. Encrypt existing user.email + compute email_hash
        result = await conn.execute(text(
            "SELECT id_user, email FROM users WHERE email IS NOT NULL AND email_hash IS NULL"
        ))
        users = result.fetchall()
        for uid, email in users:
            if email and not email.startswith("U2F"):  # skip already encrypted
                enc = encrypt_pii(email)
                h = hash_pii(email)
                await conn.execute(
                    text("UPDATE users SET email = :enc, email_hash = :h WHERE id_user = :id"),
                    {"enc": enc, "h": h, "id": uid}
                )
        print(f"Users email encrypted: {len(users)}")

        # 4. Encrypt existing guardian.email
        result = await conn.execute(text(
            "SELECT id_guardian, email FROM guardians WHERE email IS NOT NULL AND deleted_at IS NULL"
        ))
        guardians_email = result.fetchall()
        for gid, email in guardians_email:
            if email and not email.startswith("U2F"):
                enc = encrypt_pii(email)
                await conn.execute(
                    text("UPDATE guardians SET email = :enc WHERE id_guardian = :id"),
                    {"enc": enc, "id": gid}
                )
        print(f"Guardians email encrypted: {len(guardians_email)}")

        # 5. Encrypt existing guardian.phone_whatsapp + compute phone_hash
        result = await conn.execute(text(
            "SELECT id_guardian, phone_whatsapp FROM guardians WHERE phone_whatsapp IS NOT NULL AND phone_hash IS NULL AND deleted_at IS NULL"
        ))
        guardians_phone = result.fetchall()
        for gid, phone in guardians_phone:
            if phone and not phone.startswith("U2F"):
                enc = encrypt_pii(phone)
                h = hash_pii(phone)
                await conn.execute(
                    text("UPDATE guardians SET phone_whatsapp = :enc, phone_hash = :h WHERE id_guardian = :id"),
                    {"enc": enc, "h": h, "id": gid}
                )
        print(f"Guardians phone encrypted: {len(guardians_phone)}")

        # 6. Encrypt existing registration.email + phone_whatsapp
        result = await conn.execute(text(
            "SELECT id_registration, email, phone_whatsapp FROM registrations"
        ))
        regs = result.fetchall()
        for rid, email, phone in regs:
            updates = []
            params = {"id": rid}
            if email and not email.startswith("U2F"):
                updates.append("email = :enc_email")
                params["enc_email"] = encrypt_pii(email)
            if phone and not phone.startswith("U2F"):
                updates.append("phone_whatsapp = :enc_phone")
                params["enc_phone"] = encrypt_pii(phone)
            if updates:
                await conn.execute(
                    text(f"UPDATE registrations SET {', '.join(updates)} WHERE id_registration = :id"),
                    params
                )
        print(f"Registrations encrypted: {len(regs)}")

    print("\nMigration complete!")


if __name__ == "__main__":
    asyncio.run(migrate())

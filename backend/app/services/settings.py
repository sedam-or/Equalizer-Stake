"""System Settings service — runtime configuration management."""

from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.system_setting import SystemSetting

# Allowed setting keys and their schemas
ALLOWED_SETTINGS = {
    "maintenance": {
        "description": "Mode maintenance sistem",
        "fields": {"enabled": bool, "start_time": str, "end_time": str, "message": str},
    },
    "alert_thresholds": {
        "description": "Ambang batas peringatan",
        "fields": {"overdue_days": int, "max_frozen_days": int, "payment_reminder_days": int},
    },
    "whatsapp_config": {
        "description": "Konfigurasi WhatsApp Gateway",
        "fields": {"api_url": str, "api_key": str, "sender_number": str, "enabled": bool},
    },
}


async def get_all_settings(db: AsyncSession) -> list[SystemSetting]:
    """Get all system settings."""
    result = await db.execute(select(SystemSetting).order_by(SystemSetting.setting_key))
    return list(result.scalars().all())


async def get_setting(db: AsyncSession, key: str) -> SystemSetting | None:
    """Get single setting by key."""
    result = await db.execute(
        select(SystemSetting).where(SystemSetting.setting_key == key)
    )
    return result.scalar_one_or_none()


async def upsert_setting(
    db: AsyncSession,
    key: str,
    value: dict,
    updated_by: UUID,
) -> SystemSetting:
    """Create or update a setting."""
    existing = await get_setting(db, key)

    if existing:
        existing.setting_value = value
        existing.updated_by = updated_by
        return existing
    else:
        desc = ALLOWED_SETTINGS.get(key, {}).get("description", "")
        setting = SystemSetting(
            setting_key=key,
            setting_value=value,
            description=desc,
            updated_by=updated_by,
        )
        db.add(setting)
        return setting


def validate_setting_value(key: str, value: dict) -> str | None:
    """Validate setting value against schema. Returns error message or None."""
    if key not in ALLOWED_SETTINGS:
        return f"Setting key '{key}' tidak valid. Pilihan: {', '.join(ALLOWED_SETTINGS.keys())}"

    expected_fields = ALLOWED_SETTINGS[key]["fields"]
    for field_name, field_type in expected_fields.items():
        if field_name in value:
            if field_type == bool and not isinstance(value[field_name], bool):
                return f"Field '{field_name}' harus berupa boolean"
            if field_type == int and not isinstance(value[field_name], int):
                return f"Field '{field_name}' harus berupa angka"
            if field_type == str and not isinstance(value[field_name], str):
                return f"Field '{field_name}' harus berupa teks"
    return None

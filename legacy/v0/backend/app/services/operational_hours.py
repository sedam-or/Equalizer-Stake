"""Operational hours validation — Mon-Sat, 08:00-20:00.

SRS Addendum Section 10: Waktu Operasional
"""

from datetime import date, time

OPERATIONAL_DAYS = [0, 1, 2, 3, 4, 5]  # Mon=0, Sat=5, Sun=6
OPERATIONAL_START = time(8, 0)  # 08:00
OPERATIONAL_END = time(20, 0)  # 20:00


def validate_operational_day(session_date: date) -> bool:
    """Validate session is on operational day (Mon-Sat)."""
    return session_date.weekday() in OPERATIONAL_DAYS


def validate_operational_hours(start_time: time, end_time: time) -> bool:
    """Validate session is within operational hours (08:00-20:00)."""
    return start_time >= OPERATIONAL_START and end_time <= OPERATIONAL_END


def get_operational_info() -> dict:
    """Get operational hours info."""
    return {
        "days": "Senin-Sabtu",
        "start": "08:00",
        "end": "20:00",
        "closed": "Minggu",
    }

"""Duration validation — sessions must be 30 or 45 minutes.

SRS Addendum Section 11: Durasi Frekuensi
"""

from datetime import time, datetime, timedelta

VALID_DURATIONS = [30, 45]  # minutes


def validate_session_duration(start_time: time, end_time: time) -> tuple[bool, int]:
    """Validate session duration is 30 or 45 minutes.
    
    Returns: (is_valid, duration_minutes)
    """
    # Convert to datetime for calculation
    today = datetime.today().date()
    start_dt = datetime.combine(today, start_time)
    end_dt = datetime.combine(today, end_time)
    
    duration = (end_dt - start_dt).total_seconds() / 60
    
    is_valid = duration in VALID_DURATIONS
    
    return is_valid, int(duration)


def get_valid_durations() -> list[int]:
    """Get list of valid durations."""
    return VALID_DURATIONS

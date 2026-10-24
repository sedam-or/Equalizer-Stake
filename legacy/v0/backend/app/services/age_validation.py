"""Age validation — tunas class requires min 3.5 years.

SRS Addendum Section 9: Syarat Usia
"""

from datetime import date

MIN_AGE_TUNAS_YEARS = 3.5


def validate_age_for_instrument(
    date_of_birth: date | None,
    instrument: str,
) -> tuple[bool, float | None]:
    """Validate age meets minimum for instrument.
    
    Returns: (is_valid, age_years)
    """
    if instrument != "tunas":
        return True, None
    
    if date_of_birth is None:
        return False, None
    
    today = date.today()
    age_days = (today - date_of_birth).days
    age_years = age_days / 365.25
    
    is_valid = age_years >= MIN_AGE_TUNAS_YEARS
    
    return is_valid, round(age_years, 1)

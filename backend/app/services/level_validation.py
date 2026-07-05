"""Level validation per instrument — SRS Addendum Section 6."""

# Level per instrument mapping
INSTRUMENT_LEVELS: dict[str, list[str]] = {
    "piano": [
        "pra_elementary", "elementary",
        "grade_1", "grade_2", "grade_3", "grade_4", "grade_5", "grade_6",
    ],
    "biola": ["basic", "intermediate", "advance"],
    "gitar": ["basic", "intermediate", "advance"],
    "drum": ["basic", "intermediate", "advance"],
    "keyboard": ["basic", "intermediate", "advance"],
    "flute": ["basic", "intermediate", "advance"],
    "saxophone": ["basic", "intermediate", "advance"],
    "vokal": ["basic", "intermediate", "advance"],
    "bass": ["basic", "intermediate", "advance"],
    "ensemble": ["basic", "intermediate", "advance"],
}

# Flat levels for backwards compatibility (old data)
LEGACY_LEVELS = ["pemula", "dasar", "menengah", "lanjut"]


def validate_instrument_level(instrument: str, level: str) -> bool:
    """Validate level is valid for given instrument."""
    if instrument not in INSTRUMENT_LEVELS:
        return False
    return level in INSTRUMENT_LEVELS[instrument] or level in LEGACY_LEVELS


def get_valid_levels(instrument: str) -> list[str]:
    """Get valid levels for an instrument."""
    return INSTRUMENT_LEVELS.get(instrument, [])

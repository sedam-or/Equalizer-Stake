"""Extended service unit tests for coverage improvement."""

import pytest
from datetime import date, datetime, timezone
from uuid import uuid4

from app.services.whatsapp import (
    normalize_phone, is_circuit_open, get_circuit_status,
    reset_circuit_breaker, record_failure, CB_THRESHOLD,
)
from app.services.email import (
    render_template, is_circuit_open as email_cb_open,
    get_circuit_status as email_cb_status,
    reset_circuit_breaker as email_cb_reset,
    record_failure as email_record_failure,
    TEMPLATES, CB_THRESHOLD as EMAIL_CB_THRESHOLD,
)
from app.services.settings import validate_setting_value, ALLOWED_SETTINGS
from app.services.pricing import validate_pricing_input, VALID_INSTRUMENTS, VALID_LEVELS


class TestWhatsAppExtended:
    def setup_method(self):
        reset_circuit_breaker()

    def test_normalize_plus62_with_dashes(self):
        assert normalize_phone("+62-812-345-6789") == "628123456789"

    def test_normalize_already_formatted(self):
        assert normalize_phone("6281234567890") == "6281234567890"

    def test_partial_failures_below_threshold(self):
        for i in range(CB_THRESHOLD - 1):
            record_failure()
        assert is_circuit_open() is False
        assert get_circuit_status()["failures"] == CB_THRESHOLD - 1

    def test_exact_threshold(self):
        for i in range(CB_THRESHOLD):
            record_failure()
        assert is_circuit_open() is True
        assert get_circuit_status()["is_open"] is True
        assert get_circuit_status()["open_until"] is not None

    def test_multiple_resets(self):
        for _ in range(3):
            record_failure()
        reset_circuit_breaker()
        assert get_circuit_status()["failures"] == 0
        for _ in range(3):
            record_failure()
        reset_circuit_breaker()
        assert is_circuit_open() is False


class TestEmailExtended:
    def setup_method(self):
        email_cb_reset()

    def test_threshold_opens(self):
        for i in range(EMAIL_CB_THRESHOLD):
            email_record_failure()
        assert email_cb_open() is True

    def test_below_threshold(self):
        for i in range(EMAIL_CB_THRESHOLD - 1):
            email_record_failure()
        assert email_cb_open() is False

    def test_render_all_templates(self):
        vars_map = {
            "invoice": {"guardian_name": "A", "billing_period": "B", "total_amount": "C", "due_date": "D", "unique_code": "E", "items": ""},
            "reminder": {"guardian_name": "A", "student_name": "B", "session_time": "C", "room_name": "D"},
            "freeze": {"guardian_name": "A", "reason": "B"},
        }
        for name, variables in vars_map.items():
            subject, body = render_template(name, variables)
            assert len(subject) > 0
            assert len(body) > 0
            assert "<" in body  # HTML content

    def test_freeze_template_content(self):
        _, body = render_template("freeze", {"guardian_name": "Budi", "reason": "Overdue 30 hari"})
        assert "Budi" in body
        assert "Overdue 30 hari" in body
        assert "dibekukan" in body


class TestSettingsValidation:
    def test_valid_maintenance(self):
        assert validate_setting_value("maintenance", {"enabled": True}) is None

    def test_valid_alert_thresholds(self):
        assert validate_setting_value("alert_thresholds", {"overdue_days": 7}) is None

    def test_valid_whatsapp_config(self):
        assert validate_setting_value("whatsapp_config", {"api_url": "http://test", "enabled": True}) is None

    def test_invalid_key(self):
        result = validate_setting_value("bad_key", {})
        assert result is not None
        assert "tidak valid" in result

    def test_wrong_type_bool(self):
        result = validate_setting_value("maintenance", {"enabled": "yes"})
        assert result is not None
        assert "boolean" in result

    def test_wrong_type_int(self):
        result = validate_setting_value("alert_thresholds", {"overdue_days": "seven"})
        assert result is not None
        assert "angka" in result

    def test_wrong_type_str(self):
        result = validate_setting_value("whatsapp_config", {"api_url": 123})
        assert result is not None
        assert "teks" in result

    def test_allowed_settings_keys(self):
        assert "maintenance" in ALLOWED_SETTINGS
        assert "alert_thresholds" in ALLOWED_SETTINGS
        assert "whatsapp_config" in ALLOWED_SETTINGS


class TestPricingValidation:
    def test_valid_input(self):
        assert validate_pricing_input("piano", "pemula", 500000) is None

    def test_all_instruments(self):
        for inst in VALID_INSTRUMENTS:
            assert validate_pricing_input(inst, "pemula", 100) is None

    def test_all_levels(self):
        for level in VALID_LEVELS:
            assert validate_pricing_input("piano", level, 100) is None

    def test_invalid_instrument(self):
        result = validate_pricing_input("violin", "pemula", 100)
        assert result is not None
        assert "Instrumen" in result

    def test_invalid_level(self):
        result = validate_pricing_input("piano", "expert", 100)
        assert result is not None
        assert "Level" in result

    def test_negative_price(self):
        result = validate_pricing_input("piano", "pemula", -100)
        assert result is not None
        assert "negatif" in result

    def test_zero_price(self):
        assert validate_pricing_input("piano", "pemula", 0) is None

    def test_instruments_count(self):
        assert len(VALID_INSTRUMENTS) == 9

    def test_levels_count(self):
        assert len(VALID_LEVELS) == 4

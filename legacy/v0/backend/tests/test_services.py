"""Unit tests for WhatsApp and Email services."""

import pytest
from app.services.whatsapp import (
    normalize_phone,
    is_circuit_open,
    get_circuit_status,
    reset_circuit_breaker,
    record_failure,
)
from app.services.email import (
    render_template,
    is_circuit_open as email_cb_open,
    get_circuit_status as email_cb_status,
    reset_circuit_breaker as email_cb_reset,
    record_failure as email_record_failure,
    TEMPLATES,
)


class TestPhoneNormalization:
    def test_leading_zero(self):
        assert normalize_phone("08123456789") == "628123456789"

    def test_plus62(self):
        assert normalize_phone("+628123456789") == "628123456789"

    def test_already_62(self):
        assert normalize_phone("628123456789") == "628123456789"

    def test_spaces(self):
        assert normalize_phone("0812 345 6789") == "628123456789"

    def test_dashes(self):
        assert normalize_phone("0812-345-6789") == "628123456789"

    def test_empty(self):
        assert normalize_phone("") == "62"


class TestWhatsAppCircuitBreaker:
    def setup_method(self):
        reset_circuit_breaker()

    def test_default_closed(self):
        assert is_circuit_open() is False

    def test_threshold_opens(self):
        for _ in range(5):
            record_failure()
        assert is_circuit_open() is True

    def test_reset_closes(self):
        for _ in range(5):
            record_failure()
        reset_circuit_breaker()
        assert is_circuit_open() is False

    def test_status_fields(self):
        status = get_circuit_status()
        assert "failures" in status
        assert "is_open" in status
        assert "open_until" in status

    def test_below_threshold(self):
        for _ in range(4):
            record_failure()
        assert is_circuit_open() is False


class TestEmailCircuitBreaker:
    def setup_method(self):
        email_cb_reset()

    def test_default_closed(self):
        assert email_cb_open() is False

    def test_threshold_opens(self):
        for _ in range(5):
            email_record_failure()
        assert email_cb_open() is True

    def test_reset_closes(self):
        for _ in range(5):
            email_record_failure()
        email_cb_reset()
        assert email_cb_open() is False


class TestEmailTemplates:
    def test_invoice_template(self):
        subject, body = render_template("invoice", {
            "guardian_name": "Budi",
            "billing_period": "2026-07",
            "total_amount": "500.000",
            "due_date": "10 Juli 2026",
            "unique_code": "123",
            "items": "<tr><td>Andi</td><td>Piano</td><td>Rp 500.000</td></tr>",
        })
        assert "2026-07" in subject
        assert "Budi" in body
        assert "500.000" in body

    def test_reminder_template(self):
        subject, body = render_template("reminder", {
            "guardian_name": "Sari",
            "student_name": "Andi",
            "session_time": "10:00",
            "room_name": "Studio 1",
        })
        assert "Sari" in body
        assert "Andi" in body

    def test_freeze_template(self):
        subject, body = render_template("freeze", {
            "guardian_name": "Budi",
            "reason": "Tagihan overdue",
        })
        assert "dibekukan" in body
        assert "Tagihan overdue" in body

    def test_invalid_template(self):
        with pytest.raises(ValueError, match="tidak ditemukan"):
            render_template("nonexistent", {})

    def test_missing_variable_renders_empty(self):
        subject, body = render_template("invoice", {
            "guardian_name": "", "billing_period": "", "total_amount": "",
            "due_date": "", "unique_code": "", "items": "",
        })
        assert isinstance(subject, str)
        assert isinstance(body, str)

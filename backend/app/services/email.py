"""Email SMTP client — async email sending with TLS, retry, templates."""

import asyncio
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from typing import Any

import aiosmtplib
from jinja2 import BaseLoader, Environment

# Circuit breaker state for email
_cb_state: dict[str, Any] = {
    "failures": 0,
    "open_until": None,
}

CB_THRESHOLD = 5
CB_COOLDOWN = 1800  # 30 minutes


def is_circuit_open() -> bool:
    """Check if email circuit breaker is open."""
    from datetime import datetime, timedelta, timezone
    if _cb_state["open_until"]:
        if datetime.now(timezone.utc) < _cb_state["open_until"]:
            return True
        _cb_state["failures"] = 0
        _cb_state["open_until"] = None
    return False


def record_failure():
    """Record failure for circuit breaker."""
    from datetime import datetime, timedelta, timezone
    _cb_state["failures"] += 1
    if _cb_state["failures"] >= CB_THRESHOLD:
        _cb_state["open_until"] = datetime.now(timezone.utc) + timedelta(seconds=CB_COOLDOWN)


def reset_circuit_breaker():
    """Reset email circuit breaker."""
    _cb_state["failures"] = 0
    _cb_state["open_until"] = None


def get_circuit_status() -> dict:
    """Get email circuit breaker status."""
    return {
        "failures": _cb_state["failures"],
        "is_open": is_circuit_open(),
        "open_until": _cb_state["open_until"].isoformat() if _cb_state["open_until"] else None,
    }


# Email templates
TEMPLATES = {
    "invoice": {
        "subject": "Tagihan BMD Musik — {{billing_period}}",
        "body": """
<h2>Tagihan BMD Musik</h2>
<p>Kepada Yth. {{guardian_name}},</p>
<p>Berikut tagihan untuk periode <strong>{{billing_period}}</strong>:</p>
<table border="1" cellpadding="8" cellspacing="0">
  <tr><th>Siswa</th><th>Detail</th><th>Jumlah</th></tr>
  {{items}}
</table>
<p><strong>Total: Rp {{total_amount}}</strong></p>
<p>Jatuh tempo: {{due_date}}</p>
<p>Kode unik: {{unique_code}}</p>
<br><p>Salam,<br>BMD Musik Yogyakarta</p>
""",
    },
    "reminder": {
        "subject": "Pengingat Jadwal BMD Musik",
        "body": """
<h2>Pengingat Jadwal</h2>
<p>Halo {{guardian_name}},</p>
<p>Ini pengingat bahwa jadwal kursus akan dimulai dalam 1 jam:</p>
<ul>
  <li><strong>Siswa:</strong> {{student_name}}</li>
  <li><strong>Waktu:</strong> {{session_time}}</li>
  <li><strong>Ruangan:</strong> {{room_name}}</li>
</ul>
<p>Mohon hadir tepat waktu.</p>
<br><p>Salam,<br>BMD Musik Yogyakarta</p>
""",
    },
    "freeze": {
        "subject": "Pemberitahuan Pembekuan Akun — BMD Musik",
        "body": """
<h2>Pemberitahuan Pembekuan Akun</h2>
<p>Kepada Yth. {{guardian_name}},</p>
<p>Akun Anda telah <strong>dibekukan</strong> karena tagihan yang melewati jatuh tempo.</p>
<p><strong>Alasan:</strong> {{reason}}</p>
<p>Silakan hubungi admin untuk informasi lebih lanjut.</p>
<br><p>Salam,<br>BMD Musik Yogyakarta</p>
""",
    },
}


def render_template(template_name: str, variables: dict) -> tuple[str, str]:
    """Render email template with Jinja2. Returns (subject, body)."""
    tmpl = TEMPLATES.get(template_name)
    if not tmpl:
        raise ValueError(f"Template '{template_name}' tidak ditemukan")

    env = Environment(loader=BaseLoader())
    subject = env.from_string(tmpl["subject"]).render(**variables)
    body = env.from_string(tmpl["body"]).render(**variables)
    return subject, body


async def send_email(
    smtp_host: str,
    smtp_port: int,
    smtp_user: str,
    smtp_password: str,
    recipient: str,
    subject: str,
    html_body: str,
    sender_name: str = "BMD Musik",
) -> dict:
    """Send email via SMTP with TLS.
    
    Returns dict with 'success', 'error' keys.
    """
    if is_circuit_open():
        return {"success": False, "error": "Circuit breaker open", "circuit_open": True}

    msg = MIMEMultipart("alternative")
    msg["From"] = f"{sender_name} <{smtp_user}>"
    msg["To"] = recipient
    msg["Subject"] = subject
    msg.attach(MIMEText(html_body, "html", "utf-8"))

    try:
        await aiosmtplib.send(
            msg,
            hostname=smtp_host,
            port=smtp_port,
            username=smtp_user,
            password=smtp_password,
            use_tls=True,
            timeout=30,
        )

        _cb_state["failures"] = 0
        _cb_state["open_until"] = None

        return {"success": True}

    except aiosmtplib.SMTPException as e:
        record_failure()
        return {"success": False, "error": f"SMTP error: {str(e)[:200]}"}

    except Exception as e:
        record_failure()
        return {"success": False, "error": str(e)[:200]}


async def send_email_with_retry(
    smtp_host: str,
    smtp_port: int,
    smtp_user: str,
    smtp_password: str,
    recipient: str,
    subject: str,
    html_body: str,
    max_retries: int = 3,
) -> dict:
    """Send email with exponential backoff retry.
    
    Delays: 10s, 20s, 40s per SRS.
    """
    delays = [10, 20, 40]
    result: dict = {"success": False, "error": "No attempts made"}

    for attempt in range(max_retries):
        result = await send_email(
            smtp_host, smtp_port, smtp_user, smtp_password,
            recipient, subject, html_body,
        )

        if result.get("success"):
            result["retry_count"] = attempt
            return result

        if result.get("circuit_open"):
            result["retry_count"] = attempt
            return result

        if attempt < max_retries - 1:
            await asyncio.sleep(delays[attempt])

    result["retry_count"] = max_retries
    return result

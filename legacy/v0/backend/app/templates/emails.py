"""Email templates for onboarding and notifications."""


def onboarding_credentials_email(
    full_name: str,
    username: str,
    temp_password: str,
    login_url: str,
    expires_days: int = 7,
) -> tuple[str, str]:
    """Generate onboarding email subject and HTML body.

    Returns (subject, html_body).
    Actual SMTP sending is handled by notification service.
    """
    subject = "Selamat Datang di Equalizer-Stake — Akun Anda Siap"
    body = f"""<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
  <h2 style="color: #2c3e50;">Selamat Datang di Equalizer-Stake!</h2>
  <p>Halo <strong>{full_name}</strong>,</p>
  <p>Akun Anda telah dibuat oleh administrator. Berikut adalah kredensial login Anda:</p>
  <table style="border-collapse: collapse; margin: 16px 0;">
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #ddd; background: #f9f9f9;"><strong>Username</strong></td>
      <td style="padding: 8px 16px; border: 1px solid #ddd;">{username}</td>
    </tr>
    <tr>
      <td style="padding: 8px 16px; border: 1px solid #ddd; background: #f9f9f9;"><strong>Password Sementara</strong></td>
      <td style="padding: 8px 16px; border: 1px solid #ddd; font-family: monospace;">{temp_password}</td>
    </tr>
  </table>
  <p><strong>⚠️ Penting:</strong></p>
  <ul>
    <li>Password sementara ini <strong>berlaku selama {expires_days} hari</strong>.</li>
    <li>Anda <strong>wajib mengubah password</strong> saat pertama kali login.</li>
    <li>Jangan bagikan kredensial ini kepada siapapun.</li>
  </ul>
  <p>
    <a href="{login_url}"
       style="display: inline-block; padding: 12px 24px; background: #3498db; color: #fff;
              text-decoration: none; border-radius: 4px; font-weight: bold;">
      Login Sekarang
    </a>
  </p>
  <p style="color: #888; font-size: 12px; margin-top: 24px;">
    Email ini dikirim otomatis oleh sistem Equalizer-Stake. Jika Anda tidak merasa mendaftar, abaikan email ini.
  </p>
</body>
</html>"""
    return subject, body

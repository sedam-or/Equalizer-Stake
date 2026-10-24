"""PII Encryption Service — AES-256-GCM for sensitive fields.

NFR-SEC-01: PII wajib dienkripsi dengan AES-256-GCM.
UU PDP: Data pribadi (nama, alamat, foto) harus terlindungi.

Usage:
    from app.services.encryption import encrypt_pii, decrypt_pii, EncryptedString

    # Encrypt/decrypt manual
    encrypted = encrypt_pii("Budi Santoso")
    decrypted = decrypt_pii(encrypted)  # "Budi Santoso"

    # SQLAlchemy column type (auto encrypt/decrypt)
    class Guardian(Base):
        full_name = mapped_column(EncryptedString, nullable=False)
"""

import base64
import os

from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from sqlalchemy import String, TypeDecorator
from sqlalchemy.engine.interfaces import Dialect

from app.core.config import get_settings

settings = get_settings()

# Encryption key: 32 bytes for AES-256
# In production, load from env var or KMS
_ENCRYPTION_KEY: bytes | None = None


def _get_key() -> bytes:
    """Get or derive 32-byte encryption key from config."""
    global _ENCRYPTION_KEY
    if _ENCRYPTION_KEY is not None:
        return _ENCRYPTION_KEY

    raw = settings.SECRET_KEY.encode()
    # Derive 32 bytes from secret key using simple hash
    # ponnytail: use HKDF or proper KMS in production
    import hashlib
    _ENCRYPTION_KEY = hashlib.sha256(raw).digest()
    return _ENCRYPTION_KEY


def encrypt_pii(plaintext: str | None) -> str | None:
    """Encrypt PII string with AES-256-GCM.

    Returns: base64-encoded string (nonce + ciphertext + tag).
    Returns None if input is None.
    """
    if plaintext is None:
        return None

    key = _get_key()
    aesgcm = AESGCM(key)
    nonce = os.urandom(12)  # 96-bit nonce per NIST recommendation
    ciphertext = aesgcm.encrypt(nonce, plaintext.encode("utf-8"), None)
    # Prepend nonce to ciphertext for storage
    return base64.b64encode(nonce + ciphertext).decode("ascii")


def decrypt_pii(encrypted: str | None) -> str | None:
    """Decrypt PII string encrypted with AES-256-GCM.

    Returns: original plaintext string.
    Returns None if input is None.
    """
    if encrypted is None:
        return None

    key = _get_key()
    aesgcm = AESGCM(key)
    data = base64.b64decode(encrypted)
    nonce = data[:12]
    ciphertext = data[12:]
    plaintext = aesgcm.decrypt(nonce, ciphertext, None)
    return plaintext.decode("utf-8")


def is_encrypted(value: str | None) -> bool:
    """Check if a value looks like an encrypted PII string.

    Heuristic: encrypted values are base64 and longer than typical plaintext.
    """
    if value is None or len(value) < 32:
        return False
    try:
        decoded = base64.b64decode(value)
        return len(decoded) > 12  # nonce + at least some ciphertext
    except Exception:
        return False


def hash_pii(value: str | None) -> str | None:
    """Deterministic hash for blind index lookups.
    
    Returns lowercase hex SHA-256 of normalized value.
    Used for email/phone lookups on encrypted columns.
    """
    if value is None:
        return None
    import hashlib
    return hashlib.sha256(value.lower().strip().encode("utf-8")).hexdigest()


class EncryptedString(TypeDecorator):
    """SQLAlchemy column type that auto-encrypts on write and decrypts on read.

    Usage:
        class MyModel(Base):
            sensitive_field = mapped_column(EncryptedString(500), nullable=False)
    """

    impl = String
    cache_ok = True

    def __init__(self, length: int = 500, **kwargs):
        super().__init__(length=length, **kwargs)

    def process_bind_param(self, value: str | None, dialect: Dialect) -> str | None:
        """Encrypt before storing in DB."""
        if value is None:
            return None
        # Don't double-encrypt if already encrypted
        if is_encrypted(value):
            return value
        return encrypt_pii(value)

    def process_result_value(self, value: str | None, dialect: Dialect) -> str | None:
        """Decrypt when reading from DB."""
        if value is None:
            return None
        # Try to decrypt; if it fails, return as-is (legacy unencrypted data)
        try:
            return decrypt_pii(value)
        except Exception:
            return value

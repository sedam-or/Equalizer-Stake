"""Photo Storage Service — AES-256-GCM encrypted file storage.

NFR-SEC-01: PII photos wajib dienkripsi.
Stores encrypted files on disk, returns path for DB reference.
"""

import os
import uuid
import base64
from pathlib import Path
from datetime import datetime

from app.services.encryption import encrypt_pii, decrypt_pii

# Storage directories
UPLOAD_DIR = Path(os.getenv("UPLOAD_DIR", "/home/kepolu/equalizer-stake/backend/uploads"))
ATTENDANCE_DIR = UPLOAD_DIR / "attendance"
PROFILE_DIR = UPLOAD_DIR / "profiles"

# Ensure directories exist
ATTENDANCE_DIR.mkdir(parents=True, exist_ok=True)
PROFILE_DIR.mkdir(parents=True, exist_ok=True)

MAX_FILE_SIZE = 2 * 1024 * 1024  # 2MB
ALLOWED_EXTENSIONS = {".jpg", ".jpeg", ".png"}


def validate_photo(base64_data: str) -> tuple[bool, str]:
    """Validate photo base64 data. Returns (valid, error_message)."""
    if not base64_data:
        return False, "Foto tidak boleh kosong"
    
    # Check if valid base64
    try:
        # Handle data URL format
        if "," in base64_data:
            header, data = base64_data.split(",", 1)
            # Check mime type
            if "jpeg" not in header and "jpg" not in header and "png" not in header:
                return False, "Format foto harus JPEG atau PNG"
        else:
            data = base64_data
        
        raw = base64.b64decode(data)
    except Exception:
        return False, "Format foto tidak valid"
    
    # Check size
    if len(raw) > MAX_FILE_SIZE:
        return False, f"Ukuran foto maksimal {MAX_FILE_SIZE // (1024*1024)}MB"
    
    return True, ""


def save_attendance_photo(base64_data: str, session_id: str) -> str:
    """Save encrypted attendance photo. Returns file path."""
    if "," in base64_data:
        _, data = base64_data.split(",", 1)
    else:
        data = base64_data
    
    encrypted = encrypt_pii(data)
    if encrypted is None:
        raise ValueError("Gagal mengenkripsi foto")
    filename = f"{session_id}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.enc"
    filepath = ATTENDANCE_DIR / filename
    
    filepath.write_text(encrypted)
    return str(filepath)


def save_profile_photo(base64_data: str, user_id: str) -> str:
    """Save encrypted profile photo. Returns file path."""
    if "," in base64_data:
        _, data = base64_data.split(",", 1)
    else:
        data = base64_data
    
    encrypted = encrypt_pii(data)
    if encrypted is None:
        raise ValueError("Gagal mengenkripsi foto")
    filename = f"{user_id}.enc"
    filepath = PROFILE_DIR / filename
    
    filepath.write_text(encrypted)
    return str(filepath)


def load_photo(filepath: str) -> str | None:
    """Load and decrypt photo. Returns base64 data URL or None."""
    if not filepath or not os.path.exists(filepath):
        return None
    
    try:
        encrypted = Path(filepath).read_text()
        decrypted = decrypt_pii(encrypted)
        if decrypted is None:
            return None
        
        # Determine mime type from extension
        ext = Path(filepath).suffix.lower().replace(".enc", "")
        if ext in (".jpg", ".jpeg"):
            mime = "image/jpeg"
        elif ext == ".png":
            mime = "image/png"
        else:
            mime = "image/jpeg"
        
        return f"data:{mime};base64,{decrypted}"
    except Exception:
        return None


def delete_photo(filepath: str) -> bool:
    """Delete photo file."""
    if filepath and os.path.exists(filepath):
        os.remove(filepath)
        return True
    return False

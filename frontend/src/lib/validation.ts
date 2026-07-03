/**
 * Form validation utilities — Hybrid pattern (onChange + onBlur)
 * T-132: Form Validation System
 */

export type ValidationRule = {
  validate: (value: string) => boolean;
  message: string;
};

export type FieldConfig = {
  rules: ValidationRule[];
  required?: boolean;
};

export type FieldError = {
  message: string;
  valid: boolean;
};

// Validation rules
export const rules = {
  required: (msg?: string): ValidationRule => ({
    validate: (v) => v.trim().length > 0,
    message: msg || "Field ini wajib diisi",
  }),

  email: (msg?: string): ValidationRule => ({
    validate: (v) => !v || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v),
    message: msg || "Format email tidak valid",
  }),

  phone: (msg?: string): ValidationRule => ({
    validate: (v) => !v || /^(08|62)\d{8,12}$/.test(v.replace(/[\s-]/g, "")),
    message: msg || "Nomor harus 12-13 digit, awali 08 atau 62",
  }),

  minLength: (min: number, msg?: string): ValidationRule => ({
    validate: (v) => v.length >= min,
    message: msg || `Minimal ${min} karakter`,
  }),

  maxLength: (max: number, msg?: string): ValidationRule => ({
    validate: (v) => v.length <= max,
    message: msg || `Maksimal ${max} karakter`,
  }),

  number: (msg?: string): ValidationRule => ({
    validate: (v) => !v || !isNaN(Number(v)),
    message: msg || "Harus berupa angka",
  }),

  positiveNumber: (msg?: string): ValidationRule => ({
    validate: (v) => !v || (Number(v) > 0 && !isNaN(Number(v))),
    message: msg || "Harus angka positif",
  }),

  password: (msg?: string): ValidationRule => ({
    validate: (v) => v.length >= 8 && /[a-zA-Z]/.test(v) && /\d/.test(v),
    message: msg || "Minimal 8 karakter, ada huruf + angka",
  }),

  date: (msg?: string): ValidationRule => ({
    validate: (v) => !v || !isNaN(Date.parse(v)),
    message: msg || "Format tanggal tidak valid",
  }),

  match: (otherValue: string, msg?: string): ValidationRule => ({
    validate: (v) => v === otherValue,
    message: msg || "Tidak cocok",
  }),
};

// Validate a single field
export function validateField(
  value: string,
  fieldRules: ValidationRule[]
): FieldError {
  for (const rule of fieldRules) {
    if (!rule.validate(value)) {
      return { message: rule.message, valid: false };
    }
  }
  return { message: "", valid: true };
}

// Validate all fields
export function validateForm(
  values: Record<string, string>,
  schema: Record<string, ValidationRule[]>
): Record<string, FieldError> {
  const errors: Record<string, FieldError> = {};
  for (const [field, fieldRules] of Object.entries(schema)) {
    errors[field] = validateField(values[field] || "", fieldRules);
  }
  return errors;
}

// Check if form is valid
export function isFormValid(errors: Record<string, FieldError>): boolean {
  return Object.values(errors).every((e) => e.valid);
}

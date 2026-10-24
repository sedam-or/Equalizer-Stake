"use client";
/**
 * useFieldValidation hook — Hybrid validation (onChange + onBlur)
 * T-132: Form Validation System
 */

import { useState, useCallback } from "react";
import { validateField, type ValidationRule, type FieldError } from "@/lib/validation";

export function useFieldValidation(rules: ValidationRule[]) {
  const [error, setError] = useState<FieldError>({ message: "", valid: true });
  const [touched, setTouched] = useState(false);

  const validate = useCallback(
    (value: string) => {
      const result = validateField(value, rules);
      setError(result);
      return result;
    },
    [rules]
  );

  // Called on onChange — update status only if already touched
  const onChange = useCallback(
    (value: string) => {
      if (touched) {
        validate(value);
      }
    },
    [touched, validate]
  );

  // Called onBlur — validate + mark touched
  const onBlur = useCallback(
    (value: string) => {
      setTouched(true);
      validate(value);
    },
    [validate]
  );

  const reset = useCallback(() => {
    setError({ message: "", valid: true });
    setTouched(false);
  }, []);

  return { error, touched, onChange, onBlur, reset, validate };
}

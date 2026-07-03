"use client";
/**
 * Toast component — notifications (success/error/warning/info)
 * T-131: Toast Component
 */

import { useState, useEffect, useCallback } from "react";

export type ToastType = "success" | "error" | "warning" | "info";

interface Toast {
  id: string;
  type: ToastType;
  message: string;
}

let toastId = 0;
let addToastFn: ((type: ToastType, message: string) => void) | null = null;

export function toast(type: ToastType, message: string) {
  if (addToastFn) {
    addToastFn(type, message);
  }
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((type: ToastType, message: string) => {
    const id = String(++toastId);
    setToasts((prev) => [...prev, { id, type, message }]);

    // Auto-dismiss: success 3s, error 5s, others 4s
    const duration = type === "error" ? 5000 : type === "success" ? 3000 : 4000;
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, duration);
  }, []);

  useEffect(() => {
    addToastFn = addToast;
    return () => { addToastFn = null; };
  }, [addToast]);

  const dismiss = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  const icons: Record<ToastType, string> = {
    success: "✓",
    error: "✗",
    warning: "!",
    info: "i",
  };

  const colors: Record<ToastType, string> = {
    success: "border-l-4 border-green-500 bg-green-50 text-green-800",
    error: "border-l-4 border-red-500 bg-red-50 text-red-800",
    warning: "border-l-4 border-yellow-500 bg-yellow-50 text-yellow-800",
    info: "border-l-4 border-blue-500 bg-blue-50 text-blue-800",
  };

  return (
    <>
      {children}
      {/* Toast container — bottom-right desktop, top mobile */}
      <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-sm:top-4 max-sm:bottom-auto max-sm:left-4 max-sm:right-4">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={`flex items-center gap-3 rounded-lg px-4 py-3 text-sm font-medium shadow-lg animate-slide-in ${colors[t.type]}`}
          >
            <span className="text-lg font-bold">{icons[t.type]}</span>
            <span className="flex-1">{t.message}</span>
            <button
              onClick={() => dismiss(t.id)}
              className="ml-2 text-gray-400 hover:text-gray-600"
              aria-label="Tutup"
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </>
  );
}

"use client";
/**
 * ErrorState component — when API fails
 * T-140: Empty States + Loading Skeletons
 */

interface ErrorStateProps {
  message?: string;
  onRetry?: () => void;
}

export function ErrorState({
  message = "Terjadi kesalahan. Silakan coba lagi.",
  onRetry,
}: ErrorStateProps) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-red-200 bg-red-50 px-6 py-16 text-center">
      <span className="mb-4 text-5xl" aria-hidden="true">⚠️</span>
      <h3 className="font-heading text-lg font-bold text-red-800">Gagal Memuat Data</h3>
      <p className="mt-2 max-w-sm text-sm text-red-600">{message}</p>
      {onRetry && (
        <button
          onClick={onRetry}
          className="mt-6 rounded-lg bg-red-600 px-5 py-2.5 text-sm font-semibold text-white hover:bg-red-700"
        >
          Coba Lagi
        </button>
      )}
    </div>
  );
}

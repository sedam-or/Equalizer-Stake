"use client";
/**
 * EmptyState component — when no data available
 * T-140: Empty States + Loading Skeletons
 */

interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
  actionLabel?: string;
  onAction?: () => void;
  actionHref?: string;
}

export function EmptyState({
  icon = "📭",
  title,
  description,
  actionLabel,
  onAction,
  actionHref,
}: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-300 bg-white px-6 py-16 text-center">
      <span className="mb-4 text-5xl" aria-hidden="true">{icon}</span>
      <h3 className="font-heading text-lg font-bold text-gray-900">{title}</h3>
      {description && (
        <p className="mt-2 max-w-sm text-sm text-gray-500">{description}</p>
      )}
      {actionLabel && (onAction || actionHref) && (
        actionHref ? (
          <a
            href={actionHref}
            className="mt-6 rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1d4ed8]"
          >
            {actionLabel}
          </a>
        ) : (
          <button
            onClick={onAction}
            className="mt-6 rounded-lg bg-[#2563EB] px-5 py-2.5 text-sm font-semibold text-white hover:bg-[#1d4ed8]"
          >
            {actionLabel}
          </button>
        )
      )}
    </div>
  );
}

"use client";
/**
 * FilterChips component — multi-select filter chips
 * T-142: Search & Filter Components
 */

interface FilterChipsProps {
  options: string[];
  selected: string[];
  onChange: (selected: string[]) => void;
}

export function FilterChips({ options, selected, onChange }: FilterChipsProps) {
  const toggle = (option: string) => {
    if (selected.includes(option)) {
      onChange(selected.filter((s) => s !== option));
    } else {
      onChange([...selected, option]);
    }
  };

  return (
    <div className="flex flex-wrap gap-2">
      {options.map((option) => {
        const isActive = selected.includes(option);
        return (
          <button
            key={option}
            onClick={() => toggle(option)}
            className={`rounded-full px-3 py-1.5 text-xs font-medium capitalize transition-colors ${
              isActive
                ? "bg-[#2563EB] text-white"
                : "border border-gray-300 bg-white text-gray-600 hover:border-[#2563EB] hover:text-[#2563EB]"
            }`}
          >
            {option}
          </button>
        );
      })}
    </div>
  );
}

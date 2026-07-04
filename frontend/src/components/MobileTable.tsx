"use client";
/**
 * MobileTable — table on desktop, cards on mobile
 * T-151: Mobile-First Refactor
 */

interface Column {
  key: string;
  label: string;
  className?: string;
  render?: (value: any, row: any) => React.ReactNode;
}

interface MobileTableProps {
  columns: Column[];
  data: any[];
  keyField: string;
  emptyMessage?: string;
}

export function MobileTable({ columns, data, keyField, emptyMessage = "Tidak ada data" }: MobileTableProps) {
  if (data.length === 0) {
    return <p className="py-8 text-center text-sm text-gray-500">{emptyMessage}</p>;
  }

  return (
    <>
      {/* Desktop: table */}
      <div className="hidden md:block overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-200 text-left text-gray-500">
              {columns.map((col) => (
                <th key={col.key} className={`px-4 py-3 font-medium ${col.className || ""}`}>{col.label}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {data.map((row) => (
              <tr key={row[keyField]} className="border-b border-gray-100 hover:bg-gray-50">
                {columns.map((col) => (
                  <td key={col.key} className={`px-4 py-3 ${col.className || ""}`}>
                    {col.render ? col.render(row[col.key], row) : row[col.key] ?? "-"}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Mobile: cards */}
      <div className="space-y-3 md:hidden">
        {data.map((row) => (
          <div key={row[keyField]} className="rounded-xl border border-gray-200 bg-white p-4">
            {columns.map((col) => (
              <div key={col.key} className={`flex items-center justify-between py-1.5 ${col.key === columns[0].key ? "border-b border-gray-100 mb-1" : ""}`}>
                <span className="text-xs text-gray-500">{col.label}</span>
                <span className="text-sm font-medium text-gray-900">
                  {col.render ? col.render(row[col.key], row) : row[col.key] ?? "-"}
                </span>
              </div>
            ))}
          </div>
        ))}
      </div>
    </>
  );
}

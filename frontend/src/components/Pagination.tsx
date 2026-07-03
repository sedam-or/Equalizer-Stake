interface PaginationProps {
  total: number;
  limit: number;
  offset: number;
  onChange: (newOffset: number) => void;
}
export default function Pagination({ total, limit, offset, onChange }: PaginationProps) {
  const currentPage = Math.floor(offset / limit) + 1;
  const totalPages = Math.ceil(total / limit);
  if (total === 0) return null;
  return (
    <div className="flex items-center justify-between px-4 py-3 border-t border-[#CBD5E1]">
      <span className="text-sm text-gray-400">
        Menampilkan {offset + 1}-{Math.min(offset + limit, total)} dari {total}
      </span>
      <div className="flex gap-2">
        <button onClick={() => onChange(Math.max(0, offset - limit))} disabled={offset === 0}
          className="px-3 py-1 text-sm rounded border border-[#CBD5E1] text-gray-400 hover:bg-[#1a1a1a] disabled:opacity-30">
          Prev
        </button>
        <span className="px-3 py-1 text-sm text-gray-400">{currentPage}/{totalPages}</span>
        <button onClick={() => onChange(offset + limit)} disabled={offset + limit >= total}
          className="px-3 py-1 text-sm rounded border border-[#CBD5E1] text-gray-400 hover:bg-[#1a1a1a] disabled:opacity-30">
          Next
        </button>
      </div>
    </div>
  );
}

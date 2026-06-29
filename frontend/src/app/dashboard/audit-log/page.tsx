"use client";
/**
 * T-041: Audit Log Viewer (Owner + Admin)
 * UC-18: Audit Log
 * - Filter by table_name, action, date range
 * - Paginated (limit 100)
 * - Shows: table, action, changed_by, timestamp, IP
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface AuditEntry {
  id_audit: string;
  table_name: string;
  record_id: string;
  action: string;
  changed_by: string;
  created_at: string;
  ip_address: string | null;
}

const TABLE_OPTIONS = [
  "", "users", "guardians", "teachers", "students", "sessions",
  "invoices", "payments", "attendances", "rooms", "pricing",
  "notifications", "system_settings", "audit_logs",
];

const ACTION_COLORS: Record<string, string> = {
  INSERT: "bg-green-100 text-green-700",
  UPDATE: "bg-yellow-100 text-yellow-700",
  DELETE: "bg-red-100 text-red-700",
};

export default function AuditLogPage() {
  const router = useRouter();
  const [logs, setLogs] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [table, setTable] = useState("");
  const [action, setAction] = useState("");
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);

  const fetchLogs = async (reset = false) => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const newOffset = reset ? 0 : offset;
    const params = new URLSearchParams({ limit: "100", offset: String(newOffset) });
    if (table) params.set("table_name", table);

    try {
      const res = await fetch(`/api/audit?${params}`, { headers: { Authorization: `Bearer ${token}` } });
      if (res.status === 403) { router.push("/dashboard"); return; }
      if (!res.ok) { setLoading(false); return; }
      const data = await res.json();
      if (!Array.isArray(data)) { setLoading(false); return; }
      let filtered = data;
      if (action) filtered = data.filter((l: AuditEntry) => l.action === action);
      if (reset) { setLogs(filtered); setOffset(100); }
      else { setLogs(prev => [...prev, ...filtered]); setOffset(prev => prev + 100); }
      setHasMore(data.length === 100);
    } catch (e) { /* network error */ }
    finally { setLoading(false); }
  };

  useEffect(() => { fetchLogs(true); }, [table]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleFilter = () => { setLoading(true); fetchLogs(true); };

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#000000]">Audit Log</h1>
        <p className="text-sm text-gray-500">Riwayat perubahan data sistem (UC-18)</p>
      </div>

      {/* Filters */}
      <div className="bg-white rounded-xl p-4 border border-gray-200 flex flex-wrap gap-3 items-end">
        <div>
          <label className="block text-xs font-medium text-gray-500 mb-1">Tabel</label>
          <select value={table} onChange={e => setTable(e.target.value)}
            className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm">
            {TABLE_OPTIONS.map(t => <option key={t} value={t}>{t || "Semua"}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-xs font-medium text-gray-500 mb-1">Aksi</label>
          <select value={action} onChange={e => setAction(e.target.value)}
            className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm">
            <option value="">Semua</option>
            <option value="INSERT">INSERT</option>
            <option value="UPDATE">UPDATE</option>
            <option value="DELETE">DELETE</option>
          </select>
        </div>
        <button onClick={handleFilter}
          className="px-4 py-1.5 bg-[#08CB00] text-black text-sm font-medium rounded-lg hover:bg-[#06a800]">
          Filter
        </button>
        <span className="text-xs text-gray-400 ml-auto">{logs.length} entri</span>
      </div>

      {/* Log Table */}
      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Memuat audit log...</div>
        ) : logs.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Tidak ada data audit log</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Waktu</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Tabel</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Aksi</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Record ID</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">Oleh</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-600">IP</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {logs.map(log => (
                  <tr key={log.id_audit} className="hover:bg-gray-50">
                    <td className="px-4 py-2.5 whitespace-nowrap text-gray-700">
                      {new Date(log.created_at).toLocaleString("id-ID", { day: "2-digit", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit" })}
                    </td>
                    <td className="px-4 py-2.5 font-mono text-xs text-gray-600">{log.table_name}</td>
                    <td className="px-4 py-2.5">
                      <span className={`px-2 py-0.5 rounded text-xs font-medium ${ACTION_COLORS[log.action] || "bg-gray-100 text-gray-600"}`}>
                        {log.action}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 font-mono text-xs text-gray-500 truncate max-w-[120px]">{log.record_id.slice(0, 8)}...</td>
                    <td className="px-4 py-2.5 text-gray-500 text-xs truncate max-w-[100px]">{log.changed_by.slice(0, 8)}...</td>
                    <td className="px-4 py-2.5 text-gray-500 text-xs">{log.ip_address || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {hasMore && !loading && (
          <div className="p-3 border-t border-gray-100 text-center">
            <button onClick={() => fetchLogs(false)} className="text-sm text-[#08CB00] hover:underline">
              Muat Lebih Banyak
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

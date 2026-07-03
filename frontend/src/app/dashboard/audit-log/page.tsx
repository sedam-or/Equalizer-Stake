"use client";
/**
 * T-041: Audit Log Viewer (Owner + Admin)
 * UC-18: Audit Log
 */

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Pagination from "@/components/Pagination";

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
  INSERT: "bg-[#08CB00]/10 text-[#08CB00]",
  UPDATE: "bg-yellow-500/10 text-yellow-500",
  DELETE: "bg-red-500/10 text-red-400",
};

export default function AuditLogPage() {
  const router = useRouter();
  const [logs, setLogs] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [table, setTable] = useState("");
  const [action, setAction] = useState("");
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const fetchLogs = async () => {
    const token = localStorage.getItem("token");
    if (!token) { router.push("/login"); return; }
    const params = new URLSearchParams({ limit: String(limit), offset: String(offset) });
    if (table) params.set("table_name", table);

    try {
      const res = await fetch(`/api/audit?${params}`, { headers: { Authorization: `Bearer ${token}` } });
      if (res.status === 403) { router.push("/dashboard"); return; }
      if (!res.ok) { setLoading(false); return; }
      const data = await res.json();
      if (Array.isArray(data)) {
        setLogs(data);
        setTotal(data.length);
      } else {
        setLogs(data.data || []);
        setTotal(data.total || 0);
      }
    } catch {
      setLogs([]);
    }
    finally { setLoading(false); }
  };

  useEffect(() => { setLoading(true); fetchLogs(); }, [table, offset]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleFilter = () => { setLoading(true); setOffset(0); fetchLogs(); };

  const displayLogs = action ? logs.filter(l => l.action === action) : logs;

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[#EEEEEE]">Audit Log</h1>
        <p className="text-sm text-gray-400">Riwayat perubahan data sistem</p>
      </div>

      {/* Filters */}
      <div className="bg-[#111] rounded-xl p-4 border border-[#253900] flex flex-wrap gap-3 items-end">
        <div>
          <label className="block text-xs font-medium text-gray-400 mb-1">Tabel</label>
          <select value={table} onChange={e => setTable(e.target.value)}
            className="px-3 py-1.5 bg-black border border-[#253900] rounded-lg text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none">
            {TABLE_OPTIONS.map(t => <option key={t} value={t}>{t || "Semua"}</option>)}
          </select>
        </div>
        <div>
          <label className="block text-xs font-medium text-gray-400 mb-1">Aksi</label>
          <select value={action} onChange={e => setAction(e.target.value)}
            className="px-3 py-1.5 bg-black border border-[#253900] rounded-lg text-sm text-[#EEEEEE] focus:border-[#08CB00] focus:outline-none">
            <option value="">Semua</option>
            <option value="INSERT">INSERT</option>
            <option value="UPDATE">UPDATE</option>
            <option value="DELETE">DELETE</option>
          </select>
        </div>
        <button onClick={handleFilter}
          className="px-4 py-1.5 bg-[#08CB00] text-black text-sm font-semibold rounded-lg hover:bg-[#06a800]">
          Filter
        </button>
        <span className="text-xs text-gray-500 ml-auto">{displayLogs.length} entri</span>
      </div>

      {/* Log Table */}
      <div className="bg-[#111] rounded-xl border border-[#253900] overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-gray-400">Memuat audit log...</div>
        ) : displayLogs.length === 0 ? (
          <div className="p-8 text-center text-gray-400">Tidak ada data audit log</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-[#0a0a0a] border-b border-[#253900]">
                <tr>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">Waktu</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">Tabel</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">Aksi</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">Record ID</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">Oleh</th>
                  <th className="px-4 py-3 text-left font-medium text-gray-400">IP</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[#1a1a1a]">
                {displayLogs.map(log => (
                  <tr key={log.id_audit} className="hover:bg-[#1a1a1a]">
                    <td className="px-4 py-2.5 whitespace-nowrap text-gray-300">
                      {new Date(log.created_at).toLocaleString("id-ID", { day: "2-digit", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit" })}
                    </td>
                    <td className="px-4 py-2.5 font-mono text-xs text-gray-300">{log.table_name}</td>
                    <td className="px-4 py-2.5">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${ACTION_COLORS[log.action] || "bg-gray-500/10 text-gray-400"}`}>
                        {log.action}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 font-mono text-xs text-gray-400 truncate max-w-[120px]">{log.record_id.slice(0, 12)}...</td>
                    <td className="px-4 py-2.5 text-gray-400 text-xs truncate max-w-[100px]">{log.changed_by}</td>
                    <td className="px-4 py-2.5 text-gray-500 text-xs">{log.ip_address || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        <Pagination total={total} limit={limit} offset={offset} onChange={setOffset} />
      </div>
    </div>
  );
}

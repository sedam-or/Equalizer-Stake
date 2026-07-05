"use client";

import { useEffect, useState } from "react";
import { StatCard, DashboardChart, PageHeader, EmptyState } from "@/components/dashboard";
import { exportToCSV, exportToPDF } from "@/lib/export-utils";

export default function ReportsPage() {
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [period, setPeriod] = useState("2026-06");

  useEffect(() => {
    const token = localStorage.getItem("access_token");
    fetch(`/api/v1/reports/owner-dashboard?period=${period}`, {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then((r) => r.json())
      .then(setData)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [period]);

  const handleExportCSV = () => {
    if (!data) return;
    const exportData = [
      { metric: "Total Revenue", value: data.total_revenue || 0 },
      { metric: "Siswa Aktif", value: data.total_students || 0 },
      { metric: "Attendance Rate", value: `${data.attendance_rate || 0}%` },
      { metric: "Frozen Accounts", value: data.frozen_accounts || 0 },
      ...(data.instrument_distribution || []).map((i: any) => ({
        metric: `Instrumen: ${i.name}`,
        value: i.value,
      })),
      ...(data.payment_status || []).map((p: any) => ({
        metric: `Pembayaran: ${p.name}`,
        value: p.value,
      })),
    ];
    exportToCSV(exportData, `bmd-report-${period}`);
  };

  const handleExportPDF = () => {
    if (!data) return;
    const content = `
      <div class="stat">
        <div class="stat-value">Rp ${((data.total_revenue || 0) / 1000000).toFixed(1)}jt</div>
        <div class="stat-label">Total Revenue</div>
      </div>
      <div class="stat">
        <div class="stat-value">${data.total_students || 0}</div>
        <div class="stat-label">Siswa Aktif</div>
      </div>
      <div class="stat">
        <div class="stat-value">${data.attendance_rate || 0}%</div>
        <div class="stat-label">Attendance Rate</div>
      </div>
      <div class="stat">
        <div class="stat-value">${data.frozen_accounts || 0}</div>
        <div class="stat-label">Frozen Accounts</div>
      </div>

      <h2>Distribusi Instrumen</h2>
      <table>
        <tr><th>Instrumen</th><th>Jumlah Siswa</th></tr>
        ${(data.instrument_distribution || []).map((i: any) => `<tr><td>${i.name}</td><td>${i.value}</td></tr>`).join("")}
      </table>

      <h2>Status Pembayaran</h2>
      <table>
        <tr><th>Status</th><th>Jumlah</th></tr>
        ${(data.payment_status || []).map((p: any) => `<tr><td>${p.name}</td><td>${p.value}</td></tr>`).join("")}
      </table>

      <h2>Revenue Bulanan</h2>
      <table>
        <tr><th>Bulan</th><th>Revenue</th></tr>
        ${(data.monthly_revenue || []).map((r: any) => `<tr><td>${r.name}</td><td>Rp ${(r.value / 1000000).toFixed(1)}jt</td></tr>`).join("")}
      </table>
    `;
    exportToPDF(`Executive Summary — ${period}`, content);
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="h-8 w-64 animate-pulse rounded-lg bg-gray-200" />
        <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="h-24 animate-pulse rounded-xl bg-gray-200" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Laporan & Analitik"
        description="Executive summary untuk pemilik bisnis"
        action={{
          label: "Export PDF",
          href: "#",
          icon: "📄",
        }}
      />

      {/* Period Selector + Export */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <label className="text-sm text-gray-500">Periode:</label>
          <select
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
            className="rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-[#2563EB] focus:outline-none focus:ring-2 focus:ring-[#2563EB]/20"
          >
            <option value="2026-06">Juni 2026</option>
            <option value="2026-05">Mei 2026</option>
            <option value="2026-04">April 2026</option>
            <option value="2026-03">Maret 2026</option>
          </select>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleExportCSV}
            className="inline-flex items-center gap-2 rounded-lg border border-gray-300 px-4 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50"
          >
            📊 Export CSV
          </button>
          <button
            onClick={handleExportPDF}
            className="inline-flex items-center gap-2 rounded-lg bg-[#2563EB] px-4 py-2 text-sm font-semibold text-white hover:bg-[#1D4ED8]"
          >
            📄 Export PDF
          </button>
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatCard
          label="Total Revenue"
          value={`Rp ${((data?.total_revenue || 0) / 1000000).toFixed(1)}jt`}
          icon="💰"
          gradient="from-[#10B981] to-[#059669]"
          trend={{ value: 12, isPositive: true }}
        />
        <StatCard
          label="Siswa Aktif"
          value={data?.total_students || 0}
          icon="👥"
          gradient="from-[#2563EB] to-[#1E40AF]"
          trend={{ value: 5, isPositive: true }}
        />
        <StatCard
          label="Attendance Rate"
          value={`${data?.attendance_rate || 0}%`}
          icon="📊"
          gradient="from-[#F59E0B] to-[#D97706]"
        />
        <StatCard
          label="Frozen Accounts"
          value={data?.frozen_accounts || 0}
          icon="❄️"
          gradient="from-[#64748B] to-[#475569]"
        />
      </div>

      {/* Charts — 2x2 grid */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="bar"
            title="Revenue Bulanan"
            height={220}
            data={data?.monthly_revenue || [
              { name: "Jan", value: 45000000 },
              { name: "Feb", value: 52000000 },
              { name: "Mar", value: 48000000 },
              { name: "Apr", value: 55000000 },
              { name: "Mei", value: 60000000 },
              { name: "Jun", value: 58000000 },
            ]}
            colors={["#2563EB"]}
          />
        </div>
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="donut"
            title="Status Pembayaran"
            height={220}
            data={data?.payment_status || [
              { name: "Lunas", value: 85 },
              { name: "Pending", value: 10 },
              { name: "Overdue", value: 5 },
            ]}
            colors={["#10B981", "#F59E0B", "#EF4444"]}
          />
        </div>
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="line"
            title="Pertumbuhan Siswa"
            height={220}
            data={data?.student_growth || [
              { name: "Jan", value: 120 },
              { name: "Feb", value: 125 },
              { name: "Mar", value: 130 },
              { name: "Apr", value: 138 },
              { name: "Mei", value: 145 },
              { name: "Jun", value: 150 },
            ]}
            colors={["#10B981"]}
          />
        </div>
        <div style={{ height: "280px" }}>
          <DashboardChart
            type="pie"
            title="Distribusi Instrumen"
            height={220}
            data={data?.instrument_distribution || [
              { name: "Piano", value: 35 },
              { name: "Gitar", value: 28 },
              { name: "Vokal", value: 22 },
              { name: "Biola", value: 15 },
            ]}
          />
        </div>
      </div>

      {/* Summary Table */}
      <div className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
        <h3 className="mb-4 font-heading text-lg font-bold text-[#0F172A]">Ringkasan Eksekutif</h3>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          {/* Revenue Summary */}
          <div>
            <h4 className="mb-2 text-sm font-semibold text-gray-500">Revenue</h4>
            <div className="space-y-2">
              {(data?.monthly_revenue || []).slice(-3).map((r: any) => (
                <div key={r.name} className="flex items-center justify-between text-sm">
                  <span className="text-gray-600">{r.name}</span>
                  <span className="font-medium text-[#0F172A]">Rp {(r.value / 1000000).toFixed(1)}jt</span>
                </div>
              ))}
            </div>
          </div>
          {/* Payment Summary */}
          <div>
            <h4 className="mb-2 text-sm font-semibold text-gray-500">Pembayaran</h4>
            <div className="space-y-2">
              {(data?.payment_status || []).map((p: any) => (
                <div key={p.name} className="flex items-center justify-between text-sm">
                  <span className="text-gray-600">{p.name}</span>
                  <span className="font-medium text-[#0F172A]">{p.value} tagihan</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

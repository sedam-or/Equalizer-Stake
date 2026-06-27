"use client";

import { useState } from "react";

interface Invoice {
  id: string;
  period: string;
  amount: number;
  uniqueCode: string;
  status: string;
  dueDate: string;
  paidAt: string | null;
}

export default function BillingPage() {
  const [invoices] = useState<Invoice[]>([
    { id: "1", period: "Januari 2026", amount: 850000, uniqueCode: "127", status: "issued", dueDate: "2026-01-10", paidAt: null },
    { id: "2", period: "Desember 2025", amount: 850000, uniqueCode: "453", status: "paid", dueDate: "2025-12-10", paidAt: "2025-12-08" },
    { id: "3", period: "November 2025", amount: 950000, uniqueCode: "789", status: "overdue", dueDate: "2025-11-10", paidAt: null },
  ]);

  return (
    <div>
      <h1 className="text-2xl font-bold text-[#000000]">Tagihan</h1>
      <p className="mt-1 text-sm text-gray-500">Kelola tagihan dan pembayaran</p>

      {/* Summary */}
      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
        <SummaryCard label="Belum Bayar" value="Rp 850.000" color="text-yellow-600" count={1} />
        <SummaryCard label="Terlambat" value="Rp 950.000" color="text-red-500" count={1} />
        <SummaryCard label="Lunas" value="Rp 850.000" color="text-[#08CB00]" count={1} />
      </div>

      {/* Invoice list */}
      <div className="mt-6 space-y-3">
        {invoices.map((inv) => (
          <InvoiceCard key={inv.id} invoice={inv} />
        ))}
      </div>

      {/* Payment info */}
      <div className="mt-8 rounded-lg border border-[#253900] bg-[#f0fff0] p-4">
        <h3 className="font-semibold text-[#000000]">Cara Pembayaran</h3>
        <ol className="mt-2 space-y-1 text-sm text-gray-600">
          <li>1. Transfer ke rekening BMD Music</li>
          <li>2. Sertakan kode unik 3 digit (contoh: Rp 850.127)</li>
          <li>3. Kirim bukti transfer ke Admin</li>
          <li>4. Admin akan memverifikasi dalam 1×24 jam</li>
        </ol>
      </div>
    </div>
  );
}

function SummaryCard({ label, value, color, count }: {
  label: string; value: string; color: string; count: number;
}) {
  return (
    <div className="rounded-xl border border-gray-200 bg-white p-4">
      <div className="text-sm text-gray-500">{label}</div>
      <div className={`mt-1 text-xl font-bold ${color}`}>{value}</div>
      <div className="text-xs text-gray-400">{count} tagihan</div>
    </div>
  );
}

function InvoiceCard({ invoice }: { invoice: Invoice }) {
  const [showDetail, setShowDetail] = useState(false);

  const statusConfig: Record<string, { bg: string; text: string; label: string }> = {
    issued: { bg: "bg-yellow-100", text: "text-yellow-700", label: "Belum Bayar" },
    paid: { bg: "bg-[#08CB00]/10", text: "text-[#08CB00]", label: "Lunas" },
    overdue: { bg: "bg-red-100", text: "text-red-700", label: "Terlambat" },
    frozen: { bg: "bg-gray-100", text: "text-gray-700", label: "Beku" },
  };

  const status = statusConfig[invoice.status] || statusConfig.issued;

  return (
    <div className="rounded-lg border border-gray-200 bg-white">
      <div
        className="flex cursor-pointer items-center justify-between p-4"
        onClick={() => setShowDetail(!showDetail)}
      >
        <div>
          <p className="font-medium text-[#000000]">{invoice.period}</p>
          <p className="text-sm text-gray-500">Jatuh tempo: {invoice.dueDate}</p>
        </div>
        <div className="flex items-center gap-3">
          <span className={`rounded-full px-2 py-1 text-xs font-medium ${status.bg} ${status.text}`}>
            {status.label}
          </span>
          <span className="text-lg font-bold text-[#000000]">
            Rp {invoice.amount.toLocaleString("id-ID")}
          </span>
        </div>
      </div>

      {showDetail && (
        <div className="border-t border-gray-100 px-4 pb-4 pt-3">
          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span className="text-gray-500">Kode Unik:</span>
              <span className="ml-2 font-mono font-bold text-[#08CB00]">{invoice.uniqueCode}</span>
            </div>
            <div>
              <span className="text-gray-500">Status:</span>
              <span className="ml-2">{status.label}</span>
            </div>
            {invoice.paidAt && (
              <div>
                <span className="text-gray-500">Dibayar:</span>
                <span className="ml-2">{invoice.paidAt}</span>
              </div>
            )}
          </div>

          {invoice.status === "issued" && (
            <div className="mt-3 rounded-lg bg-[#f0fff0] p-3">
              <p className="text-sm text-gray-600">
                Transfer <span className="font-bold text-[#08CB00]">Rp {invoice.amount.toLocaleString("id-ID")}.{invoice.uniqueCode}</span> ke rekening BMD Music
              </p>
            </div>
          )}

          {invoice.status === "overdue" && (
            <div className="mt-3 rounded-lg bg-red-50 p-3">
              <p className="text-sm text-red-600">
                ⚠️ Denda Rp 100.000 sudah diterapkan. Akun akan dibekukan jika tidak segera dibayar.
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

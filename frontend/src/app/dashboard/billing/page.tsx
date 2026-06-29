"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";

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
  const router = useRouter();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) {
      router.push("/login");
      return;
    }

    fetch("/api/billing/invoices", {
      headers: { Authorization: `Bearer ${token}` },
    })
      .then(async (res) => {
        if (res.status === 401) {
          router.push("/login");
          return;
        }
        if (!res.ok) throw new Error("Gagal memuat tagihan");
        return res.json();
      })
      .then((data) => {
        if (data) setInvoices(Array.isArray(data) ? data : data.invoices ?? []);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [router]);

  const unpaid = invoices.filter((i) => i.status === "issued");
  const overdue = invoices.filter((i) => i.status === "overdue");
  const paid = invoices.filter((i) => i.status === "paid");

  const sumRp = (arr: Invoice[]) =>
    arr.reduce((s, i) => s + (i.amount || 0), 0);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-[#EEE]">Memuat tagihan...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-lg bg-red-900/30 p-4 text-center text-red-400">
        {error}
      </div>
    );
  }

  return (
    <div>
      <h1 className="text-2xl font-bold text-[#08CB00]">Tagihan</h1>
      <p className="mt-1 text-sm text-[#888]">Kelola tagihan dan pembayaran</p>

      {/* Summary */}
      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
        <SummaryCard
          label="Belum Bayar"
          value={`Rp ${sumRp(unpaid).toLocaleString("id-ID")}`}
          color="text-yellow-400"
          count={unpaid.length}
        />
        <SummaryCard
          label="Terlambat"
          value={`Rp ${sumRp(overdue).toLocaleString("id-ID")}`}
          color="text-red-400"
          count={overdue.length}
        />
        <SummaryCard
          label="Lunas"
          value={`Rp ${sumRp(paid).toLocaleString("id-ID")}`}
          color="text-[#08CB00]"
          count={paid.length}
        />
      </div>

      {/* Invoice list */}
      <div className="mt-6 space-y-3">
        {invoices.length === 0 ? (
          <div className="rounded-lg border border-dashed border-[#333] p-8 text-center text-[#888]">
            Tidak ada tagihan
          </div>
        ) : (
          invoices.map((inv) => <InvoiceCard key={inv.id} invoice={inv} />)
        )}
      </div>

      {/* Payment info */}
      <div className="mt-8 rounded-lg border border-[#253900] bg-[#0a1a0a] p-4">
        <h3 className="font-semibold text-[#08CB00]">Cara Pembayaran</h3>
        <ol className="mt-2 space-y-1 text-sm text-[#aaa]">
          <li>1. Transfer ke rekening BMD Music</li>
          <li>2. Sertakan kode unik 3 digit (contoh: Rp 850.127)</li>
          <li>3. Kirim bukti transfer ke Admin</li>
          <li>4. Admin akan memverifikasi dalam 1×24 jam</li>
        </ol>
      </div>
    </div>
  );
}

function SummaryCard({
  label,
  value,
  color,
  count,
}: {
  label: string;
  value: string;
  color: string;
  count: number;
}) {
  return (
    <div className="rounded-xl border border-[#222] bg-[#111] p-4">
      <div className="text-sm text-[#888]">{label}</div>
      <div className={`mt-1 text-xl font-bold ${color}`}>{value}</div>
      <div className="text-xs text-[#666]">{count} tagihan</div>
    </div>
  );
}

function InvoiceCard({ invoice }: { invoice: Invoice }) {
  const [showDetail, setShowDetail] = useState(false);

  const statusConfig: Record<
    string,
    { bg: string; text: string; label: string }
  > = {
    issued: {
      bg: "bg-yellow-900/30",
      text: "text-yellow-400",
      label: "Belum Bayar",
    },
    paid: {
      bg: "bg-[#08CB00]/10",
      text: "text-[#08CB00]",
      label: "Lunas",
    },
    overdue: {
      bg: "bg-red-900/30",
      text: "text-red-400",
      label: "Terlambat",
    },
    frozen: {
      bg: "bg-[#333]/50",
      text: "text-[#888]",
      label: "Beku",
    },
  };

  const status = statusConfig[invoice.status] || statusConfig.issued;

  return (
    <div className="rounded-lg border border-[#222] bg-[#111]">
      <div
        className="flex cursor-pointer items-center justify-between p-4"
        onClick={() => setShowDetail(!showDetail)}
      >
        <div>
          <p className="font-medium text-[#EEE]">{invoice.period}</p>
          <p className="text-sm text-[#888]">Jatuh tempo: {invoice.dueDate}</p>
        </div>
        <div className="flex items-center gap-3">
          <span
            className={`rounded-full px-2 py-1 text-xs font-medium ${status.bg} ${status.text}`}
          >
            {status.label}
          </span>
          <span className="text-lg font-bold text-[#EEE]">
            Rp {invoice.amount.toLocaleString("id-ID")}
          </span>
        </div>
      </div>

      {showDetail && (
        <div className="border-t border-[#222] px-4 pb-4 pt-3">
          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span className="text-[#888]">Kode Unik:</span>
              <span className="ml-2 font-mono font-bold text-[#08CB00]">
                {invoice.uniqueCode}
              </span>
            </div>
            <div>
              <span className="text-[#888]">Status:</span>
              <span className="ml-2 text-[#EEE]">{status.label}</span>
            </div>
            {invoice.paidAt && (
              <div>
                <span className="text-[#888]">Dibayar:</span>
                <span className="ml-2 text-[#EEE]">{invoice.paidAt}</span>
              </div>
            )}
          </div>

          {invoice.status === "issued" && (
            <div className="mt-3 rounded-lg bg-[#0a1a0a] p-3">
              <p className="text-sm text-[#aaa]">
                Transfer{" "}
                <span className="font-bold text-[#08CB00]">
                  Rp {invoice.amount.toLocaleString("id-ID")}.{invoice.uniqueCode}
                </span>{" "}
                ke rekening BMD Music
              </p>
            </div>
          )}

          {invoice.status === "overdue" && (
            <div className="mt-3 rounded-lg bg-red-900/20 p-3">
              <p className="text-sm text-red-400">
                ⚠️ Denda Rp 100.000 sudah diterapkan. Akun akan dibekukan jika
                tidak segera dibayar.
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

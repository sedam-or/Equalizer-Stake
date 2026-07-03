/**
 * Currency formatter — Indonesian Rupiah
 * T-135: Currency Formatter
 */

// Format: "Rp 350.000"
export function formatRupiah(amount: number | null | undefined): string {
  if (amount == null || isNaN(amount)) return "Rp 0";
  return "Rp " + amount.toLocaleString("id-ID");
}

// Format short: "Rp 350rb", "Rp 1,5jt", "Rp 10jt+"
export function formatRupiahShort(amount: number | null | undefined): string {
  if (amount == null || isNaN(amount)) return "Rp 0";

  if (amount >= 1_000_000_000) {
    const val = amount / 1_000_000_000;
    return `Rp ${val % 1 === 0 ? val.toFixed(0) : val.toFixed(1)}M`;
  }
  if (amount >= 1_000_000) {
    const val = amount / 1_000_000;
    return `Rp ${val % 1 === 0 ? val.toFixed(0) : val.toFixed(1)}jt`;
  }
  if (amount >= 1_000) {
    const val = amount / 1_000;
    return `Rp ${val % 1 === 0 ? val.toFixed(0) : val.toFixed(1)}rb`;
  }
  return `Rp ${amount}`;
}

// Parse "Rp 350.000" → 350000
export function parseRupiah(input: string): number {
  const cleaned = input.replace(/[Rp\s.]/g, "").replace(",", ".");
  return parseFloat(cleaned) || 0;
}

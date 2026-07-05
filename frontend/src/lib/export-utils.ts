/**
 * Export utilities for CSV and PDF
 */

export function exportToCSV(data: Record<string, unknown>[], filename: string) {
  if (data.length === 0) return;
  const headers = Object.keys(data[0]);
  const csv = [
    headers.join(","),
    ...data.map((row) =>
      headers.map((h) => {
        const val = row[h];
        // Escape commas and quotes
        if (typeof val === "string" && (val.includes(",") || val.includes('"'))) {
          return `"${val.replace(/"/g, '""')}"`;
        }
        return val;
      }).join(",")
    ),
  ].join("\n");
  const blob = new Blob(["\uFEFF" + csv], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${filename}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export function exportToPDF(title: string, content: string) {
  // Create a printable HTML document
  const html = `
    <!DOCTYPE html>
    <html>
    <head>
      <title>${title}</title>
      <style>
        body { font-family: 'Inter', sans-serif; padding: 40px; color: #0F172A; }
        h1 { font-size: 24px; font-weight: bold; margin-bottom: 8px; }
        h2 { font-size: 18px; font-weight: bold; margin-top: 24px; margin-bottom: 8px; }
        p { font-size: 14px; color: #64748B; margin-bottom: 16px; }
        table { width: 100%; border-collapse: collapse; margin: 16px 0; }
        th, td { border: 1px solid #E2E8F0; padding: 8px 12px; text-align: left; font-size: 13px; }
        th { background: #F8FAFC; font-weight: 600; }
        .stat { display: inline-block; margin-right: 24px; margin-bottom: 16px; }
        .stat-value { font-size: 24px; font-weight: bold; color: #2563EB; }
        .stat-label { font-size: 12px; color: #64748B; }
        .footer { margin-top: 32px; padding-top: 16px; border-top: 1px solid #E2E8F0; font-size: 11px; color: #94A3B8; }
        @media print { body { padding: 20px; } }
      </style>
    </head>
    <body>
      <h1>${title}</h1>
      <p>BMD Music School — Executive Summary</p>
      ${content}
      <div class="footer">
        Generated: ${new Date().toLocaleDateString("id-ID", { dateStyle: "full" })} — BMD Music School
      </div>
    </body>
    </html>
  `;
  
  const printWindow = window.open("", "_blank");
  if (printWindow) {
    printWindow.document.write(html);
    printWindow.document.close();
    printWindow.print();
  }
}

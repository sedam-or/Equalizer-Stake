/**
 * Parse API error response.
 * Pydantic returns detail as array of {type, loc, msg, input, ctx}
 * Simple errors return detail as string.
 */
export function parseApiError(data: unknown): string {
  if (!data || typeof data !== "object") return "Terjadi kesalahan";
  const obj = data as Record<string, unknown>;
  if (!obj.detail) return "Terjadi kesalahan";
  // Pydantic validation errors: array of objects
  if (Array.isArray(obj.detail)) {
    return obj.detail
      .map((e: { msg?: string; loc?: string[] }) => {
        const loc = e.loc?.join(" > ") || "";
        return loc ? `${loc}: ${e.msg}` : e.msg || "Error validasi";
      })
      .join("; ");
  }
  // Simple string error
  if (typeof obj.detail === "string") return obj.detail;
  // Object with message
  if (typeof obj.detail === "object" && obj.detail !== null && "msg" in obj.detail) {
    return (obj.detail as { msg: string }).msg;
  }
  return "Terjadi kesalahan";
}

import type { LoginResponse, UserProfile } from "@/types/api";

/**
 * Shared API client with auth interceptor + error handling.
 * Replaces direct fetch() calls across dashboard pages.
 */

const API_BASE = "/api/v1";

class ApiClient {
  private getToken(): string | null {
    return localStorage.getItem("access_token");
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    options?: { raw?: boolean }
  ): Promise<T> {
    const token = this.getToken();
    if (!token) {
      window.location.href = "/login";
      throw new Error("Tidak ada token. Silakan login.");
    }

    const headers: Record<string, string> = {
      Authorization: `Bearer ${token}`,
    };

    if (body && !(body instanceof FormData)) {
      headers["Content-Type"] = "application/json";
    }

    const res = await fetch(`${API_BASE}${path}`, {
      method,
      headers,
      body: body instanceof FormData ? body : body ? JSON.stringify(body) : undefined,
    });

    // Handle 401 → redirect to login
    if (res.status === 401) {
      localStorage.removeItem("access_token");
      localStorage.removeItem("refresh_token");
      localStorage.removeItem("role");
      window.location.href = "/login";
      throw new Error("Sesi berakhir. Silakan login kembali.");
    }

    // Handle 403
    if (res.status === 403) {
      throw new Error("Anda tidak memiliki akses ke halaman ini.");
    }

    // Handle 429
    if (res.status === 429) {
      throw new Error("Terlalu banyak permintaan. Coba beberapa saat lagi.");
    }

    // Handle 500+
    if (!res.ok) {
      const data = await res.json().catch(() => ({}));
      throw new Error(data.detail || `Gagal memuat data (${res.status})`);
    }

    if (options?.raw) {
      return res as unknown as T;
    }

    return res.json() as Promise<T>;
  }

  async get<T>(path: string): Promise<T> {
    return this.request<T>("GET", path);
  }

  async post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("POST", path, body);
  }

  async put<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("PUT", path, body);
  }

  async delete<T>(path: string): Promise<T> {
    return this.request<T>("DELETE", path);
  }
}

export const api = new ApiClient();

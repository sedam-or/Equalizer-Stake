/**
 * Shared TypeScript interfaces for API responses.
 * Maps to backend Pydantic schemas.
 */

// ─── Auth ───
export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  role: string;
}

export interface UserProfile {
  id_user: string;
  username: string;
  role: string;
  full_name?: string;
  email?: string;
  phone?: string;
  instruments?: Record<string, string>;
}

// ─── Dashboard ───
export interface AdminStats {
  greeting: string;
  date: string;
  kpi: {
    sessions_today: number;
    pending_attendance: number;
    overdue_invoices: number;
    failed_notifications: number;
  };
  recent_sessions: SessionSummary[];
  weekly_sessions: ChartDataPoint[];
  payment_status: ChartDataPoint[];
}

export interface OwnerDashboard {
  total_revenue: number;
  revenue_this_month: number;
  revenue_in_range: number;
  attendance_rate: number;
  verified_attendance: number;
  total_attendance: number;
  frozen_accounts: number;
  inactive_students: number;
  studio_utilization: number;
  sessions_this_month: number;
  total_rooms: number;
  total_students: number;
  unpaid_invoices: number;
  overdue_invoices: number;
  recent_audit: AuditLog[];
  billing_period: string;
  date_range: { start: string; end: string };
  revenue_trend: RevenueTrend[];
  studio_breakdown: StudioBreakdown[];
  anomaly_flags: string[];
  low_attendance_students: any[];
}

// ─── Sessions ───
export interface Session {
  id_session: string;
  id_student: string;
  id_teacher: string;
  id_room: string;
  session_date: string;
  start_time: string;
  end_time: string;
  session_type: string;
  status: string;
  cancellation_reason?: string;
  created_at?: string;
}

export interface SessionSummary {
  id_session: string;
  start_time: string;
  session_type: string;
  status: string;
}

export interface SlotValidation {
  available: boolean;
  conflicts: Array<{ conflict_type: string; message: string }>;
  alternatives: Array<{ start_time: string; end_time: string }>;
}

// ─── Students ───
export interface Student {
  id_student: string;
  id_guardian: string;
  full_name: string;
  instrument: string;
  level: string;
  is_active: boolean;
  created_at?: string;
}

// ─── Teachers ───
export interface Teacher {
  id_teacher: string;
  full_name: string;
  instruments: Record<string, string> | string[];
  is_active?: boolean;
}

// ─── Rooms ───
export interface Room {
  id_room: string;
  room_name: string;
  capacity?: number;
  is_active?: boolean;
}

// ─── Invoices ───
export interface Invoice {
  id_invoice: string;
  id_guardian: string;
  unique_code: string;
  billing_period: string;
  total_amount: number;
  due_date: string;
  status: string;
  issued_at?: string;
  paid_at?: string;
  guardian_name?: string;
  student_name?: string;
}

// ─── Payments ───
export interface Payment {
  id_payment: string;
  id_invoice: string;
  amount: number;
  method: string;
  payment_date: string;
  proof_url?: string;
  unique_code_used?: string;
  is_verified: boolean;
  verified_by?: string;
  verified_at?: string;
  notes?: string;
  guardian_name?: string;
  billing_period?: string;
}

// ─── Attendance ───
export interface Attendance {
  id_attendance: string;
  id_session: string;
  id_student: string;
  status: string;
  photo_url?: string;
  verified_at?: string;
  created_at?: string;
}

// ─── Charts ───
export interface ChartDataPoint {
  name: string;
  value: number;
}

export interface RevenueTrend {
  period: string;
  total_amount: number;
  invoice_count: number;
}

export interface StudioBreakdown {
  id_room: string;
  room_name: string;
  room_type: string;
  sessions: number;
  utilization_pct: number;
}

// ─── Audit ───
export interface AuditLog {
  id: string;
  table_name: string;
  action: string;
  changed_by: string;
  created_at: string;
}

// ─── Holidays ───
export interface Holiday {
  id_holiday: string;
  name: string;
  date: string;
  description?: string;
}

// ─── Notifications ───
export interface Notification {
  id_notification: string;
  title: string;
  message: string;
  status: string;
  created_at: string;
}

// ─── Pricing ───
export interface Pricing {
  id_pricing: string;
  instrument: string;
  class_type: string;
  price: number;
  monthly_fee?: number;
  registration_fee?: number;
}

// ─── Registration ───
export interface Registration {
  id_registration: string;
  full_name: string;
  phone_whatsapp: string;
  email?: string;
  status: string;
  created_at: string;
  children_data?: any;
}

// ─── Admin ───
export interface AdminUser {
  id_user: string;
  username: string;
  role: string;
  is_active: boolean;
  created_at?: string;
}

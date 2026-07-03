/**
 * Date/Time formatter — Indonesian locale
 * T-134: Date/Time Formatter
 */

import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import "dayjs/locale/id";

dayjs.extend(relativeTime);
dayjs.locale("id");

// Format: "3 Juli 2026"
export function formatDate(date: string | Date | null): string {
  if (!date) return "-";
  return dayjs(date).format("D MMMM YYYY");
}

// Format short: "3 Jul 2026"
export function formatDateShort(date: string | Date | null): string {
  if (!date) return "-";
  return dayjs(date).format("D MMM YYYY");
}

// Format: "09:00 WIB"
export function formatTime(date: string | Date | null): string {
  if (!date) return "-";
  return dayjs(date).format("HH:mm") + " WIB";
}

// Format: "3 Juli 2026, 09:00 WIB"
export function formatDateTime(date: string | Date | null): string {
  if (!date) return "-";
  return dayjs(date).format("D MMMM YYYY, HH:mm") + " WIB";
}

// Relative: "2 jam yang lalu"
export function formatRelative(date: string | Date | null): string {
  if (!date) return "-";
  return dayjs(date).fromNow();
}

// Duration: "1 jam 30 menit"
export function formatDuration(start: string | Date, end: string | Date): string {
  const diff = dayjs(end).diff(dayjs(start), "minute");
  const hours = Math.floor(diff / 60);
  const minutes = diff % 60;
  if (hours === 0) return `${minutes} menit`;
  if (minutes === 0) return `${hours} jam`;
  return `${hours} jam ${minutes} menit`;
}

// Today check
export function isToday(date: string | Date): boolean {
  return dayjs(date).isSame(dayjs(), "day");
}

// Day name: "Senin"
export function getDayName(date: string | Date): string {
  return dayjs(date).format("dddd");
}

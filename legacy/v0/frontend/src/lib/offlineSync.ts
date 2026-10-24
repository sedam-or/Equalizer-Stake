/**
 * Offline attendance sync via IndexedDB.
 * Stores submissions when offline, syncs when back online.
 * T-114: Offline Attendance Sync
 */

const DB_NAME = "eqs_offline";
const DB_VERSION = 1;
const STORE_NAME = "attendance_queue";

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(STORE_NAME, { keyPath: "id", autoIncrement: true });
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export interface OfflineAttendance {
  id_session: string;
  photo_base64: string;
  gps_lat: number;
  gps_lng: number;
  face_valid: boolean;
  face_confidence: number;
  client_timestamp: string;
  is_offline_submission: boolean;
}

/** Save attendance to IndexedDB (offline queue). */
export async function saveOfflineAttendance(data: OfflineAttendance): Promise<number> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    const req = store.add({ ...data, synced: false, created_at: new Date().toISOString() });
    req.onsuccess = () => resolve(req.result as number);
    req.onerror = () => reject(req.error);
  });
}

/** Get all pending (unsynced) attendance from IndexedDB. */
export async function getPendingAttendance(): Promise<(OfflineAttendance & { id: number })[]> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);
    const req = store.getAll();
    req.onsuccess = () => resolve((req.result as any[]).filter(r => !r.synced));
    req.onerror = () => reject(req.error);
  });
}

/** Mark attendance as synced. */
export async function markSynced(id: number): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    const getReq = store.get(id);
    getReq.onsuccess = () => {
      const record = getReq.result;
      if (record) {
        record.synced = true;
        store.put(record);
      }
      resolve();
    };
    getReq.onerror = () => reject(getReq.error);
  });
}

/** Sync all pending attendance to server. Returns count of synced items. */
export async function syncPendingAttendance(token: string): Promise<number> {
  const pending = await getPendingAttendance();
  let synced = 0;

  for (const item of pending) {
    try {
      const res = await fetch("/api/attendance/", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify({
          id_session: item.id_session,
          photo_base64: item.photo_base64,
          gps_lat: item.gps_lat,
          gps_lng: item.gps_lng,
          face_valid: item.face_valid,
          face_confidence: item.face_confidence,
          client_timestamp: item.client_timestamp,
          is_offline_submission: true,
        }),
      });
      if (res.ok) {
        await markSynced(item.id);
        synced++;
      }
    } catch {
      // Will retry on next sync
    }
  }
  return synced;
}

/** Check if online. */
export function isOnline(): boolean {
  return typeof navigator !== "undefined" ? navigator.onLine : true;
}

/** Setup auto-sync: listen for online event and sync pending. */
export function setupOfflineSync(token: string): () => void {
  const handler = async () => {
    if (navigator.onLine) {
      const count = await syncPendingAttendance(token);
      if (count > 0) {
        console.log(`[offline-sync] Synced ${count} pending attendance`);
      }
    }
  };
  window.addEventListener("online", handler);
  // Also try sync on load
  handler();
  return () => window.removeEventListener("online", handler);
}

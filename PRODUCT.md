# PRODUCT.md — Equalizer-Stake

## What is this?
Deterministic Course & Studio Reservation Engine — a web-based system for scheduling, billing, attendance, and notifications.

## Who is it for?
- **Owner**: Read-only dashboard, monitoring, audit
- **Admin Operasional**: Harian (schedule, billing, students, teachers)
- **Instructor**: Absensi, jadwal mengajar, feedback
- **Guardian**: Portal 1:N student, pembayaran, jadwal, rating

## Stack
- **Backend**: FastAPI + SQLAlchemy async + PostgreSQL + Redis
- **Frontend**: Next.js 15 (App Router) + Tailwind CSS + TypeScript
- **Deploy**: Docker + single server

## Register
App UI — design SERVES the product (dashboard, admin, portal guardian).

## Design System
- **Theme**: Light default (#F8FAFC bg, #FFFFFF cards)
- **Colors**: Primary #2563EB (blue), Accent #F59E0B (gold)
- **Typography**: Plus Jakarta Sans (headings) + Inter (body)
- **Style**: Flat + soft shadow, no glassmorphism
- **Language**: Bahasa Indonesia

## Key Routes
- `/` — Landing page (marketing)
- `/register-course` — Self-service registration (5 steps)
- `/login` — Login page
- `/dashboard` — Main dashboard (role-based)
- `/dashboard/students` — Admin: manage student
- `/dashboard/teachers` — Admin: manage instructor
- `/dashboard/schedule` — Admin: manage jadwal
- `/dashboard/billing` — Admin: manage tagihan
- `/dashboard/my-students` — Guardian: anak saya
- `/dashboard/my-schedule` — Guardian: jadwal anak
- `/dashboard/my-payments` — Guardian: pembayaran
- `/dashboard/my-schedule` — Teacher: jadwal mengajar

## Business Rules
- Auto-invoice tanggal 5 setiap bulan
- Denda Rp100.000 + freeze account tanggal 11
- Cuti max 2 bulan (60 hari) per tahun
- Durasi sesi: 30 atau 45 menit
- Jam operasional: Senin-Sabtu, 08:00-20:00
- GPS 150m radius untuk absensi

## Current Status
- 203 tasks total, 167 done (82.3%)
- Phase 30 (Business Logic Fixes) complete
- All SRS Addendum rules implemented
- Full DDL + Views + Stored Procedures + Triggers generated

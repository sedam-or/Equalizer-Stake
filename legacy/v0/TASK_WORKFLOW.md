# TASK_WORKFLOW.md — Workflow Lengkap Per Task (Equalizer-Stake)

Setiap task WAJIB mengikuti workflow ini dari awal sampai akhir. Tidak boleh skip step apapun.
Referensi SRS: `docs/SRS_Equalizer-Stake_*.md` dan `docs/part3_bab3.md`.

---

## Alur Lengkap (12 Steps)

```
┌─────────────────────────────────────────────────────────┐
│ PRE-DEBUG (4 steps) │
├─────────────────────────────────────────────────────────┤
│ 1. BACA SRS │
│ ├─ Baca srs/ modular yang relevan dengan task ini │
│ ├─ Atau docs/SRS_*.md untuk BAB utuh │
│ ├─ Catat: FR-xx, UC-xx, business rules, constraints │
│ ├─ Cek NFR (Performance, Security, Reliability) │
│ └─ Pastikan paham 100% sebelum mulai kode │
├─────────────────────────────────────────────────────────┤
│ 2. BACA DEPENDENCIES │
│ ├─ Cek task dependencies di tasks/TASKS.md │
│ ├─ Baca report sebelumnya di tasks/report_tasks.md │
│ ├─ Pastikan dependency sudah selesai │
│ └─ Cek PROJECT_RULES.md untuk constraint teknis │
│ └─ Baca architecture/ untuk technical implementation │
├─────────────────────────────────────────────────────────┤
│ 3. BACA DESIGN REFERENCE │
│ ├─ Baca design/ yang relevan (jika frontend task) │
│ ├─ Catat: warna, spacing, component pattern │
│ ├─ Mobile-first: 320px → 768px → 1024px+ │
│ └─ WCAG 2.1 Level A, bahasa Indonesia UI │
├─────────────────────────────────────────────────────────┤
│ 4. BUAT TEST PLAN │
│ ├─ Tulis: apa yang harus di-test (dari SRS criteria) │
│ ├─ Tulis: Given-When-Then acceptance criteria │
│ ├─ Tulis: edge cases yang harus dicover │
│ └─ Simpan test plan di comment sebelum kode │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ DURING-DEBUG (3 steps) │
├─────────────────────────────────────────────────────────┤
│ 5. IMPLEMENTASI │
│ ├─ Tulis kode sesuai SRS + design reference │
│ ├─ Ikuti PROJECT_RULES.md (security, audit, dll) │
│ ├─ PII wajib AES-256-GCM, audit log wajib immutable │
│ └─ Commit setiap selesai 1 subtask │
├─────────────────────────────────────────────────────────┤
│ 6. TEST LOCALHOST │
│ ├─ Run backend + frontend di localhost │
│ ├─ Test semua acceptance criteria dari test plan │
│ ├─ Test error handling (error codes AUTH_xxx, dll) │
│ └─ Pastikan semua test pass sebelum lanjut │
├─────────────────────────────────────────────────────────┤
│ 7. DEBUG LOOP │
│ ├─ Jika ada error: baca error message, fix, retest │
│ ├─ Maksimal 3 percobaan per error │
│ ├─ Jika stuck > 3x: dokumentasikan dan eskalasi │
│ └─ Loop sampai semua test pass │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ POST-DEBUG (5 steps) │
├─────────────────────────────────────────────────────────┤
│ 8. CODE REVIEW │
│ ├─ Review kode dari sisi keamanan (OWASP Top 10) │
│ ├─ Review kode dari sisi performa (NFR targets) │
│ ├─ Review kode dari sisi UU PDP compliance │
│ └─ Pastikan tidak ada hardcoded secret/credential │
├─────────────────────────────────────────────────────────┤
│ 9. OPTIMIZATION │
│ ├─ Query optimization (index, eager loading) │
│ ├─ Cache strategy (Redis untuk slot locking) │
│ ├─ Response time sesuai NFR (≤2s page, ≤1s validasi) │
│ └─ Bundle size optimization (frontend) │
├─────────────────────────────────────────────────────────┤
│ 10. SECURITY AUDIT │
│ ├─ Cek SEC-01..SEC-08 compliance matrix │
│ ├─ Input validation server-side │
│ ├─ SQL injection, XSS, CSRF protection │
│ ├─ Rate limiting pada endpoint kritis │
│ └─ JWT validation dan expiry check │
├─────────────────────────────────────────────────────────┤
│ 11. REVISION │
│ ├─ Fix semua findings dari code review + security │
│ ├─ Retest setelah fix │
│ ├─ Pastikan tidak ada regression │
│ └─ Document perubahan di commit message │
├─────────────────────────────────────────────────────────┤
│ 12. WRITE REPORT & COMMIT │
│ ├─ Tulis laporan di tasks/report_tasks.md │
│ ├─ Format: lihat template di bawah │
│ ├─ Commit semua perubahan │
│ └─ Update status di tasks/TASKS.md │
└─────────────────────────────────────────────────────────┘
```

---

## Template Report (tasks/report_tasks.md)

Gunakan format berikut untuk SETIAP task yang selesai:

```markdown
### T-XXX: [Task Title]
- **Status**: ✅ Selesai / ❌ Gagal / 🔄 In Progress
- **Tanggal**: YYYY-MM-DD
- **FR/UC terkait**: FR-xx, UC-xx
- **Files changed**:
 - `path/to/file.py` — deskripsi perubahan
 - `path/to/file.tsx` — deskripsi perubahan
- **Test results**:
 - [ ] Test case 1: PASS/FAIL
 - [ ] Test case 2: PASS/FAIL
- **Security findings**:
 - Finding 1: deskripsi + cara fix
 - Finding 2: deskripsi + cara fix
- **Performance**:
 - Response time: Xms (target: ≤2000ms)
 - Query count: N (optimized)
- **Notes**: Catatan penting, known issues, technical debt
- **Dependencies affected**: Task apa yang terdampak
- **Rollback plan**: Cara rollback jika ada masalah
```

---

## Checklist Per Step

### Step 1: BACA SRS
- [ ] Baca SRS BAB yang relevan
- [ ] Catat FR-xx dan UC-xx terkait
- [ ] Catat business rules yang berlaku
- [ ] Catat NFR targets (performance, security)
- [ ] Paham 100% sebelum mulai kode

### Step 2: BACA DEPENDENCIES
- [ ] Cek tasks/TASKS.md untuk dependencies
- [ ] Baca report dependency di tasks/report_tasks.md
- [ ] Pastikan semua dependency sudah selesai
- [ ] Cek PROJECT_RULES.md untuk constraints

### Step 3: BACA DESIGN REFERENCE
- [ ] Baca design/ yang relevan (frontend tasks)
- [ ] Catat warna, spacing, component pattern
- [ ] Mobile-first: 320px → 768px → 1024px+
- [ ] WCAG 2.1 Level A compliance

### Step 4: BUAT TEST PLAN
- [ ] Tulis test cases dari SRS acceptance criteria
- [ ] Format Given-When-Then
- [ ] Cover edge cases
- [ ] Simpan di comment sebelum kode

### Step 5: IMPLEMENTASI
- [ ] Kode sesuai SRS
- [ ] Ikuti PROJECT_RULES.md
- [ ] PII encrypted (AES-256-GCM)
- [ ] Audit log immutable
- [ ] Error messages bahasa Indonesia

### Step 6: TEST LOCALHOST
- [ ] Backend running
- [ ] Frontend running
- [ ] Semua acceptance criteria pass
- [ ] Error handling tested

### Step 7: DEBUG LOOP
- [ ] Fix semua error
- [ ] Retest setelah fix
- [ ] Maks 3 percobaan per error
- [ ] Eskalasi jika stuck

### Step 8: CODE REVIEW
- [ ] Security review (OWASP Top 10)
- [ ] Performance review (NFR targets)
- [ ] UU PDP compliance
- [ ] No hardcoded secrets

### Step 9: OPTIMIZATION
- [ ] Query optimization
- [ ] Cache strategy
- [ ] Response time OK
- [ ] Bundle size OK

### Step 10: SECURITY AUDIT
- [ ] SEC-01..SEC-08 check
- [ ] Input validation
- [ ] SQL injection protection
- [ ] Rate limiting
- [ ] JWT validation

### Step 11: REVISION
- [ ] Fix all findings
- [ ] Retest
- [ ] No regression
- [ ] Document changes

### Step 12: WRITE REPORT & COMMIT
- [ ] Report di tasks/report_tasks.md
- [ ] Format sesuai template
- [ ] Commit message jelas
- [ ] Update tasks/TASKS.md status
```

---

## Debugger Checklist

### Sebelum Mulai Task
1. Apakah SRS sudah dibaca dan dipahami?
2. Apakah semua dependency sudah selesai?
3. Apakah test plan sudah dibuat?
4. Apakah design reference sudah dibaca (frontend)?

### Selama Implementasi
1. Apakah kode sesuai SRS (FR-xx, UC-xx)?
2. Apakah PII ter-encrypt?
3. Apakah audit log immutable?
4. Apakah error messages dalam bahasa Indonesia?
5. Apakah mobile-first responsive?

### Setelah Selesai
1. Apakah semua test pass?
2. Apakah security audit clean?
3. Apakah performance sesuai NFR?
4. Apakah report sudah ditulis?
5. Apakah sudah di-commit?

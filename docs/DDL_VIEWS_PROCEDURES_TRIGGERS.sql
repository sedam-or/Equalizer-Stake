-- =====================================================
-- Equalizer-Stake — Views, Stored Procedures, Triggers
-- Generated: 2026-07-05
-- =====================================================

-- =====================================================
-- 1. VIEWS
-- =====================================================

-- View: Student dengan Guardian info
CREATE OR REPLACE VIEW v_students_with_guardian AS
SELECT 
    s.id_student,
    s.full_name AS student_name,
    s.instrument,
    s.level,
    s.enrollment_status,
    s.enrollment_date,
    s.date_of_birth,
    s.cuti_start_date,
    s.cuti_end_date,
    s.cuti_days_used,
    g.id_guardian,
    g.full_name AS guardian_name,
    g.phone_whatsapp AS guardian_phone,
    g.account_status AS guardian_status,
    u.username AS guardian_username
FROM students s
JOIN guardians g ON s.id_guardian = g.id_guardian
JOIN users u ON g.id_user = u.id_user
WHERE s.deleted_at IS NULL;

-- View: Session dengan Teacher + Student + Room info
CREATE OR REPLACE VIEW v_sessions_full AS
SELECT 
    se.id_session,
    se.session_date,
    se.start_time,
    se.end_time,
    se.session_type,
    se.status,
    t.id_teacher,
    t.full_name AS teacher_name,
    t.instruments_specialization AS teacher_instruments,
    s.id_student,
    s.full_name AS student_name,
    s.instrument AS student_instrument,
    s.level AS student_level,
    r.id_room,
    r.room_name,
    r.room_type,
    g.id_guardian,
    g.full_name AS guardian_name,
    g.phone_whatsapp AS guardian_phone
FROM sessions se
JOIN teachers t ON se.id_teacher = t.id_teacher
JOIN students s ON se.id_student = s.id_student
JOIN rooms r ON se.id_room = r.id_room
JOIN guardians g ON s.id_guardian = g.id_guardian
WHERE se.deleted_at IS NULL;

-- View: Invoice dengan Guardian info
CREATE OR REPLACE VIEW v_invoices_with_guardian AS
SELECT 
    i.id_invoice,
    i.unique_code,
    i.billing_period,
    i.total_amount,
    i.due_date,
    i.status,
    i.issued_at,
    i.paid_at,
    g.id_guardian,
    g.full_name AS guardian_name,
    g.phone_whatsapp AS guardian_phone,
    g.account_status AS guardian_status,
    (SELECT COUNT(*) FROM students st 
     WHERE st.id_guardian = g.id_guardian 
     AND st.enrollment_status = 'active'
     AND st.deleted_at IS NULL) AS active_students
FROM invoices i
JOIN guardians g ON i.id_guardian = g.id_guardian
WHERE i.deleted_at IS NULL;

-- View: Payment dengan Invoice + Guardian info
CREATE OR REPLACE VIEW v_payments_full AS
SELECT 
    p.id_payment,
    p.amount,
    p.method,
    p.payment_date,
    p.is_verified,
    p.verified_at,
    p.unique_code_used,
    i.id_invoice,
    i.billing_period,
    i.total_amount AS invoice_amount,
    g.id_guardian,
    g.full_name AS guardian_name,
    uv.username AS verified_by_username
FROM payments p
JOIN invoices i ON p.id_invoice = i.id_invoice
JOIN guardians g ON i.id_guardian = g.id_guardian
LEFT JOIN users uv ON p.verified_by = uv.id_user
WHERE p.deleted_at IS NULL;

-- View: Attendance dengan Session + Student + Teacher info
CREATE OR REPLACE VIEW v_attendance_full AS
SELECT 
    a.id_attendance,
    a.status,
    a.verification_status,
    a.photo_url,
    a.gps_lat,
    a.gps_lng,
    a.client_timestamp,
    a.reviewed_at,
    se.id_session,
    se.session_date,
    se.start_time,
    se.end_time,
    s.id_student,
    s.full_name AS student_name,
    s.instrument AS student_instrument,
    t.id_teacher,
    t.full_name AS teacher_name,
    r.room_name,
    g.id_guardian,
    g.full_name AS guardian_name,
    ur.username AS reviewed_by_username
FROM attendances a
JOIN sessions se ON a.id_session = se.id_session
JOIN students s ON a.id_student = s.id_student
JOIN teachers t ON a.id_teacher = t.id_teacher
JOIN rooms r ON se.id_room = r.id_room
JOIN guardians g ON s.id_guardian = g.id_guardian
LEFT JOIN users ur ON a.reviewed_by = ur.id_user
WHERE a.deleted_at IS NULL;

-- View: Pricing matrix
CREATE OR REPLACE VIEW v_pricing_matrix AS
SELECT 
    id_pricing,
    instrument,
    class_type,
    level,
    monthly_fee,
    registration_fee,
    duration_minutes,
    effective_date,
    is_active
FROM pricings
WHERE is_active = true
ORDER BY instrument, class_type, level;

-- View: Teacher workload
CREATE OR REPLACE VIEW v_teacher_workload AS
SELECT 
    t.id_teacher,
    t.full_name,
    t.instruments_specialization,
    t.employment_type,
    t.max_students_per_session,
    COUNT(DISTINCT se.id_session) AS total_sessions,
    COUNT(DISTINCT CASE WHEN se.session_date >= CURRENT_DATE THEN se.id_session END) AS upcoming_sessions,
    COUNT(DISTINCT se.id_student) AS unique_students
FROM teachers t
LEFT JOIN sessions se ON t.id_teacher = se.id_teacher 
    AND se.deleted_at IS NULL
WHERE t.deleted_at IS NULL
GROUP BY t.id_teacher, t.full_name, t.instruments_specialization, 
         t.employment_type, t.max_students_per_session;

-- View: Guardian summary
CREATE OR REPLACE VIEW v_guardian_summary AS
SELECT 
    g.id_guardian,
    g.full_name,
    g.phone_whatsapp,
    g.account_status,
    u.username,
    COUNT(DISTINCT s.id_student) AS total_students,
    COUNT(DISTINCT CASE WHEN s.enrollment_status = 'active' THEN s.id_student END) AS active_students,
    COUNT(DISTINCT i.id_invoice) AS total_invoices,
    COUNT(DISTINCT CASE WHEN i.status = 'paid' THEN i.id_invoice END) AS paid_invoices,
    COUNT(DISTINCT CASE WHEN i.status = 'overdue' THEN i.id_invoice END) AS overdue_invoices,
    COALESCE(SUM(CASE WHEN i.status = 'overdue' THEN i.total_amount ELSE 0 END), 0) AS overdue_amount
FROM guardians g
JOIN users u ON g.id_user = u.id_user
LEFT JOIN students s ON g.id_guardian = s.id_guardian AND s.deleted_at IS NULL
LEFT JOIN invoices i ON g.id_guardian = i.id_guardian AND i.deleted_at IS NULL
WHERE g.deleted_at IS NULL
GROUP BY g.id_guardian, g.full_name, g.phone_whatsapp, g.account_status, u.username;

-- View: Monthly revenue
CREATE OR REPLACE VIEW v_monthly_revenue AS
SELECT 
    billing_period,
    COUNT(*) AS total_invoices,
    SUM(total_amount) AS total_billed,
    SUM(CASE WHEN status = 'paid' THEN total_amount ELSE 0 END) AS total_paid,
    SUM(CASE WHEN status = 'overdue' THEN total_amount ELSE 0 END) AS total_overdue,
    ROUND(
        SUM(CASE WHEN status = 'paid' THEN total_amount ELSE 0 END) * 100.0 / 
        NULLIF(SUM(total_amount), 0), 2
    ) AS collection_rate
FROM invoices
WHERE deleted_at IS NULL
GROUP BY billing_period
ORDER BY billing_period DESC;


-- =====================================================
-- 2. STORED PROCEDURES (FUNCTIONS)
-- =====================================================

-- Function: Generate unique code for invoice
CREATE OR REPLACE FUNCTION fn_generate_unique_code(
    p_billing_period VARCHAR(7),
    p_guardian_id UUID
) RETURNS CHAR(3) AS $$
DECLARE
    v_code CHAR(3);
    v_exists BOOLEAN;
BEGIN
    LOOP
        -- Generate random 3-digit code
        v_code := LPAD(FLOOR(RANDOM() * 1000)::TEXT, 3, '0');
        
        -- Check if code exists for this billing period
        SELECT EXISTS(
            SELECT 1 FROM invoices 
            WHERE unique_code = v_code 
            AND billing_period = p_billing_period
            AND deleted_at IS NULL
        ) INTO v_exists;
        
        EXIT WHEN NOT v_exists;
    END LOOP;
    
    RETURN v_code;
END;
$$ LANGUAGE plpgsql;

-- Function: Calculate student age
CREATE OR REPLACE FUNCTION fn_calculate_age(
    p_date_of_birth DATE
) RETURNS NUMERIC(4,1) AS $$
BEGIN
    RETURN ROUND((CURRENT_DATE - p_date_of_birth) / 365.25, 1);
END;
$$ LANGUAGE plpgsql;

-- Function: Check if student eligible for exam
CREATE OR REPLACE FUNCTION fn_check_exam_eligibility(
    p_student_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    v_attendance_count INTEGER;
    v_total_sessions INTEGER;
    v_attendance_rate NUMERIC;
BEGIN
    -- Count attended sessions
    SELECT COUNT(*) INTO v_attendance_count
    FROM attendances a
    WHERE a.id_student = p_student_id
    AND a.status = 'hadir'
    AND a.verification_status = 'verified'
    AND a.deleted_at IS NULL;
    
    -- Count total sessions
    SELECT COUNT(*) INTO v_total_sessions
    FROM sessions s
    WHERE s.id_student = p_student_id
    AND s.status = 'completed'
    AND s.deleted_at IS NULL;
    
    -- Calculate attendance rate
    IF v_total_sessions = 0 THEN
        RETURN FALSE;
    END IF;
    
    v_attendance_rate := v_attendance_count * 100.0 / v_total_sessions;
    
    -- Eligible if attendance rate >= 80%
    RETURN v_attendance_rate >= 80;
END;
$$ LANGUAGE plpgsql;

-- Function: Get guardian invoice summary
CREATE OR REPLACE FUNCTION fn_guardian_invoice_summary(
    p_guardian_id UUID,
    p_billing_period VARCHAR(7) DEFAULT NULL
) RETURNS TABLE(
    total_invoices BIGINT,
    total_billed NUMERIC,
    total_paid NUMERIC,
    overdue_count BIGINT,
    overdue_amount NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COUNT(*) AS total_invoices,
        COALESCE(SUM(i.total_amount), 0) AS total_billed,
        COALESCE(SUM(CASE WHEN i.status = 'paid' THEN i.total_amount ELSE 0 END), 0) AS total_paid,
        COUNT(CASE WHEN i.status = 'overdue' THEN 1 END) AS overdue_count,
        COALESCE(SUM(CASE WHEN i.status = 'overdue' THEN i.total_amount ELSE 0 END), 0) AS overdue_amount
    FROM invoices i
    WHERE i.id_guardian = p_guardian_id
    AND i.deleted_at IS NULL
    AND (p_billing_period IS NULL OR i.billing_period = p_billing_period);
END;
$$ LANGUAGE plpgsql;

-- Function: Auto-invoice generation
CREATE OR REPLACE FUNCTION fn_generate_monthly_invoices()
RETURNS INTEGER AS $$
DECLARE
    v_guardian RECORD;
    v_billing_period VARCHAR(7);
    v_total_amount NUMERIC;
    v_unique_code CHAR(3);
    v_invoice_count INTEGER := 0;
BEGIN
    -- Current billing period (previous month)
    v_billing_period := TO_CHAR(CURRENT_DATE - INTERVAL '1 month', 'YYYY-MM');
    
    -- Loop through active guardians with active students
    FOR v_guardian IN 
        SELECT DISTINCT g.id_guardian
        FROM guardians g
        JOIN students s ON g.id_guardian = s.id_guardian
        WHERE g.account_status = 'active'
        AND s.enrollment_status = 'active'
        AND s.deleted_at IS NULL
        AND g.deleted_at IS NULL
    LOOP
        -- Calculate total amount for guardian's students
        SELECT COALESCE(SUM(p.monthly_fee), 0) INTO v_total_amount
        FROM students s
        JOIN pricings p ON s.instrument = p.instrument
        WHERE s.id_guardian = v_guardian.id_guardian
        AND s.enrollment_status = 'active'
        AND s.deleted_at IS NULL
        AND p.is_active = true;
        
        -- Skip if no amount
        IF v_total_amount = 0 THEN
            CONTINUE;
        END IF;
        
        -- Generate unique code
        v_unique_code := fn_generate_unique_code(v_billing_period, v_guardian.id_guardian);
        
        -- Create invoice
        INSERT INTO invoices (
            id_invoice, id_guardian, unique_code, billing_period,
            total_amount, due_date, status, issued_at
        ) VALUES (
            gen_random_uuid(), v_guardian.id_guardian, v_unique_code, v_billing_period,
            v_total_amount, CURRENT_DATE + INTERVAL '10 days', 'issued', NOW()
        );
        
        v_invoice_count := v_invoice_count + 1;
    END LOOP;
    
    RETURN v_invoice_count;
END;
$$ LANGUAGE plpgsql;


-- =====================================================
-- 3. TRIGGERS
-- =====================================================

-- Trigger: Update updated_at on row change
CREATE OR REPLACE FUNCTION trg_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to all tables with updated_at
CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_guardians_updated_at
    BEFORE UPDATE ON guardians
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_students_updated_at
    BEFORE UPDATE ON students
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_teachers_updated_at
    BEFORE UPDATE ON teachers
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_invoices_updated_at
    BEFORE UPDATE ON invoices
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_payments_updated_at
    BEFORE UPDATE ON payments
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_attendances_updated_at
    BEFORE UPDATE ON attendances
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

CREATE TRIGGER trg_pricings_updated_at
    BEFORE UPDATE ON pricings
    FOR EACH ROW EXECUTE FUNCTION trg_update_timestamp();

-- Trigger: Update invoice status when payment verified
CREATE OR REPLACE FUNCTION trg_update_invoice_on_payment()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_verified = true AND OLD.is_verified = false THEN
        UPDATE invoices 
        SET status = 'paid', paid_at = NOW()
        WHERE id_invoice = NEW.id_invoice;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_payment_verified
    AFTER UPDATE ON payments
    FOR EACH ROW EXECUTE FUNCTION trg_update_invoice_on_payment();

-- Trigger: Update student exam eligibility
CREATE OR REPLACE FUNCTION trg_update_exam_eligibility()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.verification_status = 'verified' AND NEW.status = 'hadir' THEN
        UPDATE students 
        SET eligible_for_exam = fn_check_exam_eligibility(NEW.id_student)
        WHERE id_student = NEW.id_student;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_attendance_verified
    AFTER UPDATE ON attendances
    FOR EACH ROW EXECUTE FUNCTION trg_update_exam_eligibility();

-- Trigger: Log audit on student changes
CREATE OR REPLACE FUNCTION trg_audit_student()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit_logs (id_audit, user_id, action, entity_type, entity_id, after_state, created_at)
        VALUES (gen_random_uuid(), NULL, 'INSERT', 'student', NEW.id_student, 
                jsonb_build_object('name', NEW.full_name, 'instrument', NEW.instrument, 'level', NEW.level), NOW());
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_logs (id_audit, user_id, action, entity_type, entity_id, before_state, after_state, created_at)
        VALUES (gen_random_uuid(), NULL, 'UPDATE', 'student', NEW.id_student,
                jsonb_build_object('name', OLD.full_name, 'status', OLD.enrollment_status),
                jsonb_build_object('name', NEW.full_name, 'status', NEW.enrollment_status), NOW());
    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO audit_logs (id_audit, user_id, action, entity_type, entity_id, before_state, created_at)
        VALUES (gen_random_uuid(), NULL, 'DELETE', 'student', OLD.id_student,
                jsonb_build_object('name', OLD.full_name, 'instrument', OLD.instrument), NOW());
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_student_audit
    AFTER INSERT OR UPDATE OR DELETE ON students
    FOR EACH ROW EXECUTE FUNCTION trg_audit_student();

-- Trigger: Freeze guardian on overdue invoice
CREATE OR REPLACE FUNCTION trg_freeze_on_overdue()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'overdue' AND OLD.status != 'overdue' THEN
        UPDATE guardians 
        SET account_status = 'frozen', frozen_at = NOW(), frozen_reason = 'Invoice overdue: ' || NEW.id_invoice
        WHERE id_guardian = NEW.id_guardian
        AND account_status = 'active';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_invoice_overdue
    AFTER UPDATE ON invoices
    FOR EACH ROW EXECUTE FUNCTION trg_freeze_on_overdue();

-- Trigger: Validate session duration
CREATE OR REPLACE FUNCTION trg_validate_session_duration()
RETURNS TRIGGER AS $$
DECLARE
    v_duration INTEGER;
BEGIN
    v_duration := EXTRACT(EPOCH FROM (NEW.end_time - NEW.start_time)) / 60;
    
    IF v_duration NOT IN (30, 45) THEN
        RAISE EXCEPTION 'Session duration must be 30 or 45 minutes, got % minutes', v_duration;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_session_duration_check
    BEFORE INSERT OR UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION trg_validate_session_duration();

-- Trigger: Validate operational hours
CREATE OR REPLACE FUNCTION trg_validate_operational_hours()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if Sunday (weekday 0 in PostgreSQL)
    IF EXTRACT(DOW FROM NEW.session_date) = 0 THEN
        RAISE EXCEPTION 'Studio is closed on Sundays';
    END IF;
    
    -- Check operational hours (08:00-20:00)
    IF NEW.start_time < '08:00:00' OR NEW.end_time > '20:00:00' THEN
        RAISE EXCEPTION 'Operational hours are 08:00-20:00';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_session_operational_check
    BEFORE INSERT OR UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION trg_validate_operational_hours();

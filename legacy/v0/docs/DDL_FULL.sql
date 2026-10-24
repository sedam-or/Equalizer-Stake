-- Equalizer-Stake — Full DDL
-- Generated from SQLAlchemy models
-- Total tables: 18

-- Table: users

CREATE TABLE users (
	id_user UUID NOT NULL, 
	username VARCHAR(50) NOT NULL, 
	email VARCHAR(500) NOT NULL, 
	email_hash VARCHAR(64), 
	password_hash VARCHAR(255) NOT NULL, 
	role user_role NOT NULL, 
	is_active BOOLEAN NOT NULL, 
	last_login_at TIMESTAMP WITH TIME ZONE, 
	refresh_token TEXT, 
	token_expires_at TIMESTAMP WITH TIME ZONE, 
	failed_login_attempts INTEGER NOT NULL, 
	locked_until TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	first_login BOOLEAN NOT NULL, 
	is_temp_password BOOLEAN NOT NULL, 
	password_expires_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_user), 
	UNIQUE (username), 
	UNIQUE (email_hash)
)


;

-- Table: guardians

CREATE TABLE guardians (
	id_guardian UUID NOT NULL, 
	id_user UUID NOT NULL, 
	full_name VARCHAR(500) NOT NULL, 
	phone_whatsapp VARCHAR(500) NOT NULL, 
	phone_hash VARCHAR(64), 
	email VARCHAR(500), 
	address VARCHAR(1000), 
	account_status account_status NOT NULL, 
	frozen_at TIMESTAMP WITH TIME ZONE, 
	frozen_reason TEXT, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_guardian), 
	UNIQUE (id_user), 
	FOREIGN KEY(id_user) REFERENCES users (id_user), 
	UNIQUE (phone_hash)
)


;

-- Table: students

CREATE TABLE students (
	id_student UUID NOT NULL, 
	id_guardian UUID NOT NULL, 
	full_name VARCHAR(500) NOT NULL, 
	date_of_birth DATE, 
	instrument instrument_type NOT NULL, 
	level VARCHAR(50) NOT NULL, 
	enrollment_status enrollment_status NOT NULL, 
	enrollment_date DATE NOT NULL, 
	trial_end_date DATE, 
	eligible_for_exam BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_student), 
	FOREIGN KEY(id_guardian) REFERENCES guardians (id_guardian)
)


;

-- Table: teachers

CREATE TABLE teachers (
	id_teacher UUID NOT NULL, 
	id_user UUID NOT NULL, 
	full_name VARCHAR(255) NOT NULL, 
	instruments_specialization JSONB NOT NULL, 
	employment_type employment_type NOT NULL, 
	max_students_per_session INTEGER NOT NULL, 
	is_active BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_teacher), 
	UNIQUE (id_user), 
	FOREIGN KEY(id_user) REFERENCES users (id_user)
)


;

-- Table: rooms

CREATE TABLE rooms (
	id_room UUID NOT NULL, 
	room_name VARCHAR(100) NOT NULL, 
	room_type room_type NOT NULL, 
	capacity INTEGER NOT NULL, 
	is_available BOOLEAN NOT NULL, 
	location_notes TEXT, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_room), 
	UNIQUE (room_name)
)


;

-- Table: sessions

CREATE TABLE sessions (
	id_session UUID NOT NULL, 
	id_teacher UUID NOT NULL, 
	id_student UUID NOT NULL, 
	id_room UUID NOT NULL, 
	session_date DATE NOT NULL, 
	start_time TIME WITHOUT TIME ZONE NOT NULL, 
	end_time TIME WITHOUT TIME ZONE NOT NULL, 
	session_type session_type NOT NULL, 
	status session_status NOT NULL, 
	rescheduled_to UUID, 
	cancellation_reason TEXT, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_session), 
	CONSTRAINT uq_session_room_date_start UNIQUE (id_room, session_date, start_time), 
	FOREIGN KEY(id_teacher) REFERENCES teachers (id_teacher), 
	FOREIGN KEY(id_student) REFERENCES students (id_student), 
	FOREIGN KEY(id_room) REFERENCES rooms (id_room), 
	FOREIGN KEY(rescheduled_to) REFERENCES sessions (id_session)
)


;

-- Table: invoices

CREATE TABLE invoices (
	id_invoice UUID NOT NULL, 
	id_guardian UUID NOT NULL, 
	unique_code CHAR(3) NOT NULL, 
	billing_period VARCHAR(7) NOT NULL, 
	total_amount NUMERIC(12, 2) NOT NULL, 
	due_date DATE NOT NULL, 
	status invoice_status NOT NULL, 
	issued_at TIMESTAMP WITH TIME ZONE, 
	paid_at TIMESTAMP WITH TIME ZONE, 
	notes TEXT, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_invoice), 
	FOREIGN KEY(id_guardian) REFERENCES guardians (id_guardian)
)


;

-- Table: payments

CREATE TABLE payments (
	id_payment UUID NOT NULL, 
	id_invoice UUID NOT NULL, 
	amount NUMERIC(12, 2) NOT NULL, 
	method payment_method NOT NULL, 
	payment_date TIMESTAMP WITH TIME ZONE NOT NULL, 
	proof_url TEXT, 
	unique_code_used CHAR(3) NOT NULL, 
	is_verified BOOLEAN NOT NULL, 
	verified_by UUID, 
	verified_at TIMESTAMP WITH TIME ZONE, 
	notes TEXT, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_payment), 
	FOREIGN KEY(id_invoice) REFERENCES invoices (id_invoice), 
	FOREIGN KEY(verified_by) REFERENCES users (id_user)
)


;

-- Table: attendances

CREATE TABLE attendances (
	id_attendance UUID NOT NULL, 
	id_session UUID NOT NULL, 
	id_teacher UUID NOT NULL, 
	photo_url VARCHAR(1000) NOT NULL, 
	gps_lat NUMERIC(10, 7) NOT NULL, 
	gps_lng NUMERIC(10, 7) NOT NULL, 
	gps_distance_m FLOAT NOT NULL, 
	face_valid BOOLEAN NOT NULL, 
	face_confidence FLOAT, 
	client_timestamp TIMESTAMP WITH TIME ZONE NOT NULL, 
	server_timestamp TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	verification_status verification_status NOT NULL, 
	rejection_reason TEXT, 
	reviewed_by UUID, 
	reviewed_at TIMESTAMP WITH TIME ZONE, 
	is_offline_submission BOOLEAN NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_attendance), 
	FOREIGN KEY(id_session) REFERENCES sessions (id_session), 
	FOREIGN KEY(id_teacher) REFERENCES teachers (id_teacher), 
	FOREIGN KEY(reviewed_by) REFERENCES users (id_user)
)


;

-- Table: notifications

CREATE TABLE notifications (
	id_notification UUID NOT NULL, 
	recipient_id UUID NOT NULL, 
	channel notif_channel NOT NULL, 
	category notif_category NOT NULL, 
	subject VARCHAR(255), 
	body TEXT NOT NULL, 
	status notif_status NOT NULL, 
	retry_count INTEGER NOT NULL, 
	error_message TEXT, 
	sent_at TIMESTAMP WITH TIME ZONE, 
	related_entity_type VARCHAR(50), 
	related_entity_id UUID, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_notification), 
	FOREIGN KEY(recipient_id) REFERENCES users (id_user)
)


;

-- Table: audit_logs

CREATE TABLE audit_logs (
	id_audit UUID NOT NULL, 
	table_name TEXT NOT NULL, 
	record_id UUID NOT NULL, 
	action audit_action NOT NULL, 
	before_state JSONB, 
	after_state JSONB, 
	changed_by UUID, 
	details TEXT, 
	ip_address INET, 
	user_agent TEXT, 
	category audit_category, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_audit), 
	FOREIGN KEY(changed_by) REFERENCES users (id_user)
)


;

-- Table: system_settings

CREATE TABLE system_settings (
	id_setting UUID NOT NULL, 
	setting_key VARCHAR(100) NOT NULL, 
	setting_value JSONB NOT NULL, 
	description TEXT, 
	updated_by UUID, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_setting)
)


;

-- Table: pricings

CREATE TABLE pricings (
	id_pricing UUID NOT NULL, 
	instrument VARCHAR(50) NOT NULL, 
	class_type VARCHAR(50) NOT NULL, 
	level VARCHAR(50) NOT NULL, 
	monthly_fee NUMERIC(12, 2) NOT NULL, 
	duration_minutes INTEGER NOT NULL, 
	registration_fee NUMERIC(12, 2) NOT NULL, 
	effective_date DATE NOT NULL, 
	description TEXT, 
	is_active BOOLEAN NOT NULL, 
	created_by UUID, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_pricing)
)


;

-- Table: drafts

CREATE TABLE drafts (
	id_draft UUID NOT NULL, 
	id_user UUID NOT NULL, 
	form_key VARCHAR(100) NOT NULL, 
	draft_data JSONB NOT NULL, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_draft), 
	CONSTRAINT uq_user_form_key UNIQUE (id_user, form_key)
)


;

-- Table: registrations

CREATE TABLE registrations (
	id_registration UUID NOT NULL, 
	full_name VARCHAR(255) NOT NULL, 
	phone_whatsapp VARCHAR(500) NOT NULL, 
	email VARCHAR(500), 
	address TEXT, 
	instrument VARCHAR(50) NOT NULL, 
	level VARCHAR(50) NOT NULL, 
	class_type VARCHAR(50) NOT NULL, 
	schedule_id VARCHAR(50), 
	payment_proof_path VARCHAR(500), 
	children_data JSONB, 
	status registration_status NOT NULL, 
	reviewed_by UUID, 
	reviewed_at TIMESTAMP WITH TIME ZONE, 
	notes TEXT, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_registration), 
	FOREIGN KEY(reviewed_by) REFERENCES users (id_user)
)


;

-- Table: deletion_requests

CREATE TABLE deletion_requests (
	id_request UUID NOT NULL, 
	id_user UUID NOT NULL, 
	entity_type deletion_entity_type NOT NULL, 
	entity_id UUID NOT NULL, 
	reason TEXT NOT NULL, 
	status deletion_status NOT NULL, 
	admin_notes TEXT, 
	processed_by UUID, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	processed_at TIMESTAMP WITH TIME ZONE, 
	PRIMARY KEY (id_request)
)


;

-- Table: feedbacks

CREATE TABLE feedbacks (
	id_feedback UUID NOT NULL, 
	id_session UUID NOT NULL, 
	id_teacher UUID NOT NULL, 
	id_guardian UUID NOT NULL, 
	rating INTEGER NOT NULL, 
	comment TEXT, 
	category feedback_category NOT NULL, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_feedback)
)


;

-- Table: ratings

CREATE TABLE ratings (
	id_rating UUID NOT NULL, 
	id_teacher UUID NOT NULL, 
	id_guardian UUID NOT NULL, 
	rating INTEGER NOT NULL, 
	comment TEXT, 
	deleted_at TIMESTAMP WITH TIME ZONE, 
	created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL, 
	PRIMARY KEY (id_rating)
)


;

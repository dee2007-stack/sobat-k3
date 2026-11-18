-- Migration 0001: Inisialisasi Skema Sobat K3 (BBPOM di Bandung)

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(160) NOT NULL UNIQUE,
    full_name VARCHAR(160) NOT NULL,
    nip VARCHAR(64) NOT NULL DEFAULT '',
    position_title VARCHAR(160) NOT NULL DEFAULT '',
    role VARCHAR(32) NOT NULL CHECK (role IN ('admin', 'ketua_tim_k3', 'sekretaris_tim_k3', 'anggota_tim_k3')),
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS document_periods (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    year INT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    subtitle TEXT NOT NULL,
    doc_number VARCHAR(120) NOT NULL,
    revision VARCHAR(64) NOT NULL,
    sk_basis TEXT NOT NULL,
    legal_basis TEXT NOT NULL,
    approval_date_text VARCHAR(120) NOT NULL DEFAULT 'Bandung, .................... 2026',
    chair_name VARCHAR(160) NOT NULL DEFAULT '',
    chair_nip VARCHAR(64) NOT NULL DEFAULT '',
    chair_signed_at TIMESTAMPTZ,
    secretary_name VARCHAR(160) NOT NULL DEFAULT '',
    secretary_nip VARCHAR(64) NOT NULL DEFAULT '',
    secretary_verified_at TIMESTAMPTZ,
    head_name VARCHAR(160) NOT NULL DEFAULT '',
    head_nip VARCHAR(64) NOT NULL DEFAULT '',
    head_approved_at TIMESTAMPTZ,
    status VARCHAR(32) NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS work_hours_monthly (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    period_id UUID NOT NULL REFERENCES document_periods(id) ON DELETE CASCADE,
    month INT NOT NULL CHECK (month BETWEEN 1 AND 12),
    effective_days DOUBLE PRECISION NOT NULL DEFAULT 0,
    standard_hours_per_day DOUBLE PRECISION NOT NULL DEFAULT 8,
    avg_employees_present DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (period_id, month)
);

CREATE TABLE IF NOT EXISTS work_programs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    period_id UUID NOT NULL REFERENCES document_periods(id) ON DELETE CASCADE,
    pillar VARCHAR(8) NOT NULL CHECK (pillar IN ('A', 'B', 'C', 'D')),
    pillar_title TEXT NOT NULL,
    subcategory TEXT NOT NULL DEFAULT '',
    item_number VARCHAR(16) NOT NULL DEFAULT '',
    objective TEXT NOT NULL DEFAULT '',
    measurable_target TEXT NOT NULL DEFAULT '',
    code VARCHAR(32) NOT NULL,
    description TEXT NOT NULL,
    frequency VARCHAR(120) NOT NULL DEFAULT '',
    reference_law TEXT NOT NULL DEFAULT '',
    pic TEXT NOT NULL DEFAULT '',
    output_evidence TEXT NOT NULL DEFAULT '',
    annual_target DOUBLE PRECISION NOT NULL DEFAULT 0,
    is_target_auto_sum BOOLEAN NOT NULL DEFAULT FALSE,
    root_cause_analysis TEXT NOT NULL DEFAULT '',
    corrective_preventive_action TEXT NOT NULL DEFAULT '',
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS work_program_monthly (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    program_id UUID NOT NULL REFERENCES work_programs(id) ON DELETE CASCADE,
    month INT NOT NULL CHECK (month BETWEEN 1 AND 12),
    plan_label VARCHAR(64) NOT NULL DEFAULT '',
    plan_value DOUBLE PRECISION NOT NULL DEFAULT 0,
    actual_value DOUBLE PRECISION NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (program_id, month)
);

CREATE TABLE IF NOT EXISTS evidence_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    program_id UUID NOT NULL REFERENCES work_programs(id) ON DELETE CASCADE,
    month INT CHECK (month IS NULL OR (month BETWEEN 1 AND 12)),
    file_name VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    mime_type VARCHAR(120) NOT NULL DEFAULT 'application/octet-stream',
    file_size BIGINT NOT NULL DEFAULT 0,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS lagging_indicators (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    period_id UUID NOT NULL REFERENCES document_periods(id) ON DELETE CASCADE,
    item_number INT NOT NULL,
    parameter_name TEXT NOT NULL,
    indicator_key VARCHAR(64) NOT NULL,
    frequency VARCHAR(120) NOT NULL DEFAULT 'Bulanan',
    reference_formula TEXT NOT NULL DEFAULT '',
    pic TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    max_target DOUBLE PRECISION NOT NULL DEFAULT 0,
    is_auto_calculated BOOLEAN NOT NULL DEFAULT FALSE,
    root_cause_analysis TEXT NOT NULL DEFAULT '',
    corrective_action TEXT NOT NULL DEFAULT '',
    UNIQUE (period_id, item_number)
);

CREATE TABLE IF NOT EXISTS lagging_monthly (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    indicator_id UUID NOT NULL REFERENCES lagging_indicators(id) ON DELETE CASCADE,
    month INT NOT NULL CHECK (month BETWEEN 1 AND 12),
    value DOUBLE PRECISION NOT NULL DEFAULT 0,
    lost_days INT NOT NULL DEFAULT 0,
    incident_notes TEXT NOT NULL DEFAULT '',
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (indicator_id, month)
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    username VARCHAR(64) NOT NULL DEFAULT 'system',
    action VARCHAR(64) NOT NULL,
    entity_type VARCHAR(64) NOT NULL,
    entity_id VARCHAR(64),
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

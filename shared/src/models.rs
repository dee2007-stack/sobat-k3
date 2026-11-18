use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    KetuaTimK3,
    SekretarisTimK3,
    AnggotaTimK3,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::KetuaTimK3 => "ketua_tim_k3",
            UserRole::SekretarisTimK3 => "sekretaris_tim_k3",
            UserRole::AnggotaTimK3 => "anggota_tim_k3",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UserRole::Admin => "Admin",
            UserRole::KetuaTimK3 => "Ketua Tim K3",
            UserRole::SekretarisTimK3 => "Sekretaris Tim K3",
            UserRole::AnggotaTimK3 => "Anggota Tim K3",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(UserRole::Admin),
            "ketua_tim_k3" => Some(UserRole::KetuaTimK3),
            "sekretaris_tim_k3" => Some(UserRole::SekretarisTimK3),
            "anggota_tim_k3" => Some(UserRole::AnggotaTimK3),
            _ => None,
        }
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn can_manage_master(&self) -> bool {
        matches!(
            self,
            UserRole::Admin | UserRole::KetuaTimK3 | UserRole::SekretarisTimK3
        )
    }

    pub fn can_input_realization(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub nip: String,
    pub position_title: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub nip: String,
    pub position_title: String,
    pub role: UserRole,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: String,
    pub full_name: String,
    pub nip: String,
    pub position_title: String,
    pub role: UserRole,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogDto {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPeriodDto {
    pub id: Uuid,
    pub year: i32,
    pub title: String,
    pub subtitle: String,
    pub doc_number: String,
    pub revision: String,
    pub sk_basis: String,
    pub legal_basis: String,
    pub approval_date_text: String,
    pub chair_name: String,
    pub chair_nip: String,
    pub chair_signed_at: Option<DateTime<Utc>>,
    pub secretary_name: String,
    pub secretary_nip: String,
    pub secretary_verified_at: Option<DateTime<Utc>>,
    pub head_name: String,
    pub head_nip: String,
    pub head_approved_at: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePeriodRequest {
    pub title: String,
    pub subtitle: String,
    pub doc_number: String,
    pub revision: String,
    pub sk_basis: String,
    pub legal_basis: String,
    pub approval_date_text: String,
    pub chair_name: String,
    pub chair_nip: String,
    pub secretary_name: String,
    pub secretary_nip: String,
    pub head_name: String,
    pub head_nip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignPeriodRequest {
    pub stage: String, // "secretary" | "chair" | "head"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHoursMonthDto {
    pub id: Uuid,
    pub period_id: Uuid,
    pub month: i32,
    pub effective_days: f64,
    pub standard_hours_per_day: f64,
    pub avg_employees_present: f64,
    pub total_daily_hours: f64,
    pub cumulative_monthly_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkHoursMonthRequest {
    pub effective_days: f64,
    pub standard_hours_per_day: f64,
    pub avg_employees_present: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHoursSummaryDto {
    pub months: Vec<WorkHoursMonthDto>,
    pub total_effective_days: f64,
    pub avg_standard_hours: f64,
    pub avg_employees_present: f64,
    pub sum_daily_hours: f64,
    pub total_cumulative_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProgramMonthDto {
    pub month: i32,
    pub plan_label: String,
    pub plan_value: f64,
    pub actual_value: f64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceFileDto {
    pub id: Uuid,
    pub program_id: Uuid,
    pub month: Option<i32>,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProgramDto {
    pub id: Uuid,
    pub period_id: Uuid,
    pub pillar: String,
    pub pillar_title: String,
    pub subcategory: String,
    pub item_number: String,
    pub objective: String,
    pub measurable_target: String,
    pub code: String,
    pub description: String,
    pub frequency: String,
    pub reference_law: String,
    pub pic: String,
    pub output_evidence: String,
    pub annual_target: f64,
    pub annual_realization: f64,
    pub achievement_pct: f64,
    pub is_target_auto_sum: bool,
    pub root_cause_analysis: String,
    pub corrective_preventive_action: String,
    pub sort_order: i32,
    pub months: Vec<WorkProgramMonthDto>,
    pub evidences: Vec<EvidenceFileDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertWorkProgramRequest {
    pub pillar: String,
    pub pillar_title: String,
    pub subcategory: String,
    pub item_number: String,
    pub objective: String,
    pub measurable_target: String,
    pub code: String,
    pub description: String,
    pub frequency: String,
    pub reference_law: String,
    pub pic: String,
    pub output_evidence: String,
    pub annual_target: f64,
    pub is_target_auto_sum: bool,
    pub root_cause_analysis: String,
    pub corrective_preventive_action: String,
    pub sort_order: i32,
    pub months: Vec<WorkProgramMonthDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgramMonthRequest {
    pub plan_label: Option<String>,
    pub plan_value: Option<f64>,
    pub actual_value: f64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaggingMonthDto {
    pub month: i32,
    pub value: f64,
    pub lost_days: i32,
    pub incident_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaggingIndicatorDto {
    pub id: Uuid,
    pub period_id: Uuid,
    pub item_number: i32,
    pub parameter_name: String,
    pub indicator_key: String,
    pub frequency: String,
    pub reference_formula: String,
    pub pic: String,
    pub notes: String,
    pub max_target: f64,
    pub annual_realization: f64,
    pub compliance_status: String,
    pub is_auto_calculated: bool,
    pub root_cause_analysis: String,
    pub corrective_action: String,
    pub months: Vec<LaggingMonthDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLaggingMetaRequest {
    pub root_cause_analysis: String,
    pub corrective_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLaggingMonthRequest {
    pub value: f64,
    pub lost_days: i32,
    pub incident_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarSummaryDto {
    pub pillar: String,
    pub title: String,
    pub total_programs: usize,
    pub completed_programs: usize,
    pub avg_achievement_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummaryDto {
    pub period: DocumentPeriodDto,
    pub safe_manhours: f64,
    pub total_effective_days: f64,
    pub fr: f64,
    pub sr: f64,
    pub trir: f64,
    pub total_fatalities: f64,
    pub total_lti: f64,
    pub total_nearmiss: f64,
    pub overall_achievement_pct: f64,
    pub pillars: Vec<PillarSummaryDto>,
    pub under_target_programs: Vec<WorkProgramDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

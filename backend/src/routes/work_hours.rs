use crate::auth::{AuthUser, MasterGuard};
use crate::db::log_audit;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, put, State};
use serde_json::json;
use shared::formulas::{calculate_daily_hours, calculate_monthly_cumulative_hours};
use shared::models::{
    ApiError, UpdateWorkHoursMonthRequest, WorkHoursMonthDto, WorkHoursSummaryDto,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
struct WorkHourRow {
    id: Uuid,
    period_id: Uuid,
    month: i32,
    effective_days: f64,
    standard_hours_per_day: f64,
    avg_employees_present: f64,
}

pub async fn fetch_work_hours_summary(
    pool: &PgPool,
    period_id: Uuid,
) -> Result<WorkHoursSummaryDto, sqlx::Error> {
    let rows = sqlx::query_as::<_, WorkHourRow>(
        "SELECT id, period_id, month, effective_days, standard_hours_per_day, avg_employees_present
         FROM work_hours_monthly WHERE period_id = $1 ORDER BY month ASC",
    )
    .bind(period_id)
    .fetch_all(pool)
    .await?;

    let mut months = Vec::with_capacity(rows.len());
    let mut total_effective_days = 0.0;
    let mut sum_std_hours = 0.0;
    let mut sum_avg_employees = 0.0;
    let mut sum_daily_hours = 0.0;
    let mut total_cumulative_hours = 0.0;

    for r in rows {
        let daily = calculate_daily_hours(r.standard_hours_per_day, r.avg_employees_present);
        let cum = calculate_monthly_cumulative_hours(
            r.effective_days,
            r.standard_hours_per_day,
            r.avg_employees_present,
        );
        total_effective_days += r.effective_days;
        sum_std_hours += r.standard_hours_per_day;
        sum_avg_employees += r.avg_employees_present;
        sum_daily_hours += daily;
        total_cumulative_hours += cum;

        months.push(WorkHoursMonthDto {
            id: r.id,
            period_id: r.period_id,
            month: r.month,
            effective_days: r.effective_days,
            standard_hours_per_day: r.standard_hours_per_day,
            avg_employees_present: r.avg_employees_present,
            total_daily_hours: daily,
            cumulative_monthly_hours: cum,
        });
    }

    let count = if months.is_empty() {
        1.0
    } else {
        months.len() as f64
    };

    Ok(WorkHoursSummaryDto {
        months,
        total_effective_days,
        avg_standard_hours: sum_std_hours / count,
        avg_employees_present: sum_avg_employees / count,
        sum_daily_hours,
        total_cumulative_hours,
    })
}

#[get("/api/periods/<id>/work-hours")]
pub async fn get_work_hours(
    pool: &State<PgPool>,
    _auth: AuthUser,
    id: &str,
) -> Result<Json<WorkHoursSummaryDto>, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let summary = fetch_work_hours_summary(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(summary))
}

#[put("/api/periods/<id>/work-hours/<month>", data = "<req>")]
pub async fn update_work_hours_month(
    pool: &State<PgPool>,
    master: MasterGuard,
    id: &str,
    month: i32,
    req: Json<UpdateWorkHoursMonthRequest>,
) -> Result<Json<WorkHoursSummaryDto>, (Status, Json<ApiError>)> {
    if !(1..=12).contains(&month) {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                message: "Bulan harus antara 1 dan 12".to_string(),
            }),
        ));
    }

    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    sqlx::query(
        "UPDATE work_hours_monthly
         SET effective_days = $1, standard_hours_per_day = $2, avg_employees_present = $3, updated_at = NOW()
         WHERE period_id = $4 AND month = $5",
    )
    .bind(req.effective_days.max(0.0))
    .bind(req.standard_hours_per_day.max(0.0))
    .bind(req.avg_employees_present.max(0.0))
    .bind(period_id)
    .bind(month)
    .execute(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    log_audit(
        pool.inner(),
        Some(master.0.id),
        &master.0.username,
        "UPDATE_WORK_HOURS",
        "work_hours_monthly",
        Some(&format!("{}:{}", id, month)),
        json!({
            "effective_days": req.effective_days,
            "avg_employees_present": req.avg_employees_present
        }),
    )
    .await;

    let summary = fetch_work_hours_summary(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(Json(summary))
}

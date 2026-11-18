use crate::auth::AuthUser;
use crate::routes::lagging::fetch_lagging_for_period;
use crate::routes::periods::PeriodRow;
use crate::routes::programs::fetch_programs_for_period;
use crate::routes::work_hours::fetch_work_hours_summary;
use rocket::http::{ContentType, Header, Status};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{get, Request, Response, State};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};
use shared::models::{ApiError, DashboardSummaryDto, PillarSummaryDto};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

#[get("/api/periods/<id>/dashboard")]
pub async fn get_dashboard(
    pool: &State<PgPool>,
    _auth: AuthUser,
    id: &str,
) -> Result<Json<DashboardSummaryDto>, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let period = sqlx::query_as::<_, PeriodRow>(
        "SELECT id, year, title, subtitle, doc_number, revision, sk_basis, legal_basis,
                approval_date_text, chair_name, chair_nip, chair_signed_at,
                secretary_name, secretary_nip, secretary_verified_at,
                head_name, head_nip, head_approved_at, status
         FROM document_periods WHERE id = $1",
    )
    .bind(period_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?
    .ok_or((
        Status::NotFound,
        Json(ApiError {
            message: "Periode tidak ditemukan".to_string(),
        }),
    ))?
    .into_dto();

    let wh = fetch_work_hours_summary(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let programs = fetch_programs_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let lagging = fetch_lagging_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let val_of = |key: &str| -> f64 {
        lagging
            .iter()
            .find(|l| l.indicator_key == key)
            .map(|l| l.annual_realization)
            .unwrap_or(0.0)
    };

    let pillar_defs = [
        ("A", "Keselamatan Kerja (Safety)"),
        ("B", "Kesehatan Kerja & Sanitair (Health)"),
        ("C", "Lingkungan Hidup & Limbah B3 (Environment)"),
        ("D", "Mutu Layanan & K3 Inklusif"),
    ];

    let mut pillars = Vec::new();
    for (code, title) in pillar_defs {
        let p_list: Vec<_> = programs.iter().filter(|p| p.pillar == code).collect();
        let total = p_list.len();
        let completed = p_list.iter().filter(|p| p.achievement_pct >= 100.0).count();
        let avg = if total > 0 {
            p_list.iter().map(|p| p.achievement_pct.min(100.0)).sum::<f64>() / total as f64
        } else {
            0.0
        };
        pillars.push(PillarSummaryDto {
            pillar: code.to_string(),
            title: title.to_string(),
            total_programs: total,
            completed_programs: completed,
            avg_achievement_pct: avg,
        });
    }

    let overall_achievement_pct = if !programs.is_empty() {
        programs
            .iter()
            .map(|p| p.achievement_pct.min(100.0))
            .sum::<f64>()
            / programs.len() as f64
    } else {
        0.0
    };

    let under_target_programs = programs
        .into_iter()
        .filter(|p| p.achievement_pct < 100.0)
        .take(8)
        .collect();

    Ok(Json(DashboardSummaryDto {
        period,
        safe_manhours: wh.total_cumulative_hours,
        total_effective_days: wh.total_effective_days,
        fr: val_of("fr"),
        sr: val_of("sr"),
        trir: val_of("trir"),
        total_fatalities: val_of("fatality"),
        total_lti: val_of("lti"),
        total_nearmiss: val_of("nearmiss"),
        overall_achievement_pct,
        pillars,
        under_target_programs,
    }))
}

pub struct XlsxFile {
    pub bytes: Vec<u8>,
    pub filename: String,
}

impl<'r> Responder<'r, 'static> for XlsxFile {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .header(ContentType::new(
                "application",
                "vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ))
            .header(Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", self.filename),
            ))
            .sized_body(self.bytes.len(), Cursor::new(self.bytes))
            .ok()
    }
}

#[get("/api/periods/<id>/export/xlsx")]
pub async fn export_xlsx(
    pool: &State<PgPool>,
    _auth: AuthUser,
    id: &str,
) -> Result<XlsxFile, (Status, Json<ApiError>)> {
    let period_id = Uuid::parse_str(id).map_err(|_| {
        (
            Status::BadRequest,
            Json(ApiError {
                message: "ID periode tidak valid".to_string(),
            }),
        )
    })?;

    let period = sqlx::query_as::<_, PeriodRow>(
        "SELECT id, year, title, subtitle, doc_number, revision, sk_basis, legal_basis,
                approval_date_text, chair_name, chair_nip, chair_signed_at,
                secretary_name, secretary_nip, secretary_verified_at,
                head_name, head_nip, head_approved_at, status
         FROM document_periods WHERE id = $1",
    )
    .bind(period_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?
    .into_dto();

    let wh = fetch_work_hours_summary(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let programs = fetch_programs_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;
    let lagging = fetch_lagging_for_period(pool.inner(), period_id)
        .await
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    let mut workbook = Workbook::new();
    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x1E3A8A))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    let cell_fmt = Format::new().set_border(FormatBorder::Thin);
    let num_fmt = Format::new()
        .set_border(FormatBorder::Thin)
        .set_num_format("#,##0.##");

    // Sheet 1: Jam Kerja Efektif
    let ws1 = workbook.add_worksheet();
    let _ = ws1.set_name("1. Jam Kerja Efektif");
    let _ = ws1.write_string_with_format(0, 0, &period.title, &header_fmt);
    let months = ["Jan","Feb","Mar","Apr","Mei","Jun","Jul","Agt","Sep","Okt","Nov","Des"];
    let _ = ws1.write_string_with_format(2, 0, "Parameter Perhitungan", &header_fmt);
    for (i, m) in months.iter().enumerate() {
        let _ = ws1.write_string_with_format(2, (i + 1) as u16, *m, &header_fmt);
    }
    let _ = ws1.write_string_with_format(2, 13, "Total / Kumulatif", &header_fmt);

    let _ = ws1.write_string_with_format(3, 0, "Hari kerja efektif ASN per bulan (hari)", &cell_fmt);
    let _ = ws1.write_string_with_format(4, 0, "Standar jam kerja efektif per hari (jam)", &cell_fmt);
    let _ = ws1.write_string_with_format(5, 0, "Rata-rata jumlah pegawai hadir per hari (orang)", &cell_fmt);
    let _ = ws1.write_string_with_format(6, 0, "Total jam kerja seluruh pegawai per hari (jam)", &cell_fmt);
    let _ = ws1.write_string_with_format(7, 0, "Total jam kerja kumulatif per bulan (jam)", &cell_fmt);

    for m in &wh.months {
        let col = m.month as u16;
        let col_letter = ((b'A' + col as u8) as char).to_string();
        let _ = ws1.write_number_with_format(3, col, m.effective_days, &num_fmt);
        let _ = ws1.write_number_with_format(4, col, m.standard_hours_per_day, &num_fmt);
        let _ = ws1.write_number_with_format(5, col, m.avg_employees_present, &num_fmt);
        let _ = ws1.write_formula_with_format(
            6,
            col,
            format!("={}5*{}6", col_letter, col_letter).as_str(),
            &num_fmt,
        );
        let _ = ws1.write_formula_with_format(
            7,
            col,
            format!("={}4*{}7", col_letter, col_letter).as_str(),
            &num_fmt,
        );
    }
    let _ = ws1.write_formula_with_format(3, 13, "=SUM(B4:M4)", &num_fmt);
    let _ = ws1.write_formula_with_format(4, 13, "=AVERAGE(B5:M5)", &num_fmt);
    let _ = ws1.write_formula_with_format(5, 13, "=AVERAGE(B6:M6)", &num_fmt);
    let _ = ws1.write_formula_with_format(6, 13, "=SUM(B7:M7)", &num_fmt);
    let _ = ws1.write_formula_with_format(7, 13, "=SUM(B8:M8)", &num_fmt);

    // Sheet 2: Program Kerja TSP
    let ws2 = workbook.add_worksheet();
    let _ = ws2.set_name("2. Matriks TSP P2K3");
    let cols = [
        "Pilar", "No", "Tujuan", "Sasaran Terukur", "Kode", "Uraian Program Kerja",
        "Frekuensi", "Referensi", "PIC", "Jan", "Feb", "Mar", "Apr", "Mei", "Jun",
        "Jul", "Agt", "Sep", "Okt", "Nov", "Des", "Output", "Target", "Realisasi",
        "Capaian (%)", "Akar Masalah", "Tindakan Perbaikan",
    ];
    for (c, title) in cols.iter().enumerate() {
        let _ = ws2.write_string_with_format(0, c as u16, *title, &header_fmt);
    }
    for (r_idx, p) in programs.iter().enumerate() {
        let row = (r_idx + 1) as u32;
        let _ = ws2.write_string_with_format(row, 0, &p.pillar, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 1, &p.item_number, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 2, &p.objective, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 3, &p.measurable_target, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 4, &p.code, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 5, &p.description, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 6, &p.frequency, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 7, &p.reference_law, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 8, &p.pic, &cell_fmt);
        for m in &p.months {
            let col = (8 + m.month) as u16;
            let text = if m.actual_value > 0.0 {
                format!("{} (Akt: {})", m.plan_label, m.actual_value)
            } else {
                m.plan_label.clone()
            };
            let _ = ws2.write_string_with_format(row, col, &text, &cell_fmt);
        }
        let excel_row = row + 1;
        let _ = ws2.write_string_with_format(row, 21, &p.output_evidence, &cell_fmt);
        let _ = ws2.write_number_with_format(row, 22, p.annual_target, &num_fmt);
        let _ = ws2.write_number_with_format(row, 23, p.annual_realization, &num_fmt);
        let _ = ws2.write_formula_with_format(
            row,
            24,
            format!("=IFERROR((X{}/W{})*100,0)", excel_row, excel_row).as_str(),
            &num_fmt,
        );
        let _ = ws2.write_string_with_format(row, 25, &p.root_cause_analysis, &cell_fmt);
        let _ = ws2.write_string_with_format(row, 26, &p.corrective_preventive_action, &cell_fmt);
    }

    // Sheet 3: Lagging Indicators
    let ws3 = workbook.add_worksheet();
    let _ = ws3.set_name("3. Lagging Indicators");
    let l_cols = [
        "No", "Parameter", "Frekuensi", "Referensi/Rumus", "PIC",
        "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agt", "Sep", "Okt", "Nov", "Des",
        "Target Maks", "Realisasi", "Status Kepatuhan", "Akar Masalah", "Tindakan Korektif",
    ];
    for (c, t) in l_cols.iter().enumerate() {
        let _ = ws3.write_string_with_format(0, c as u16, *t, &header_fmt);
    }
    for (r_idx, l) in lagging.iter().enumerate() {
        let row = (r_idx + 1) as u32;
        let excel_row = row + 1;
        let _ = ws3.write_number_with_format(row, 0, l.item_number as f64, &num_fmt);
        let _ = ws3.write_string_with_format(row, 1, &l.parameter_name, &cell_fmt);
        let _ = ws3.write_string_with_format(row, 2, &l.frequency, &cell_fmt);
        let _ = ws3.write_string_with_format(row, 3, &l.reference_formula, &cell_fmt);
        let _ = ws3.write_string_with_format(row, 4, &l.pic, &cell_fmt);
        for m in &l.months {
            let col = (4 + m.month) as u16;
            let col_letter = ((b'A' + col as u8) as char).to_string();
            let wh_col = ((b'A' + m.month as u8) as char).to_string();
            if l.indicator_key == "safe_manhours" {
                let _ = ws3.write_formula_with_format(
                    row,
                    col,
                    format!("='1. Jam Kerja Efektif'!{}8", wh_col).as_str(),
                    &num_fmt,
                );
            } else if l.indicator_key == "fr" {
                let _ = ws3.write_formula_with_format(
                    row,
                    col,
                    format!("=IFERROR(({}3*1000000)/{}13,0)", col_letter, col_letter).as_str(),
                    &num_fmt,
                );
            } else if l.indicator_key == "trir" {
                let _ = ws3.write_formula_with_format(
                    row,
                    col,
                    format!(
                        "=IFERROR((({}2+{}3+{}4+{}5)*200000)/{}13,0)",
                        col_letter, col_letter, col_letter, col_letter, col_letter
                    )
                    .as_str(),
                    &num_fmt,
                );
            } else {
                let _ = ws3.write_number_with_format(row, col, m.value, &num_fmt);
            }
        }
        let _ = ws3.write_number_with_format(row, 17, l.max_target, &num_fmt);
        if l.indicator_key == "fr" {
            let _ = ws3.write_formula_with_format(row, 18, "=IFERROR((S3*1000000)/S13,0)", &num_fmt);
        } else if l.indicator_key == "trir" {
            let _ = ws3.write_formula_with_format(
                row,
                18,
                "=IFERROR(((S2+S3+S4+S5)*200000)/S13,0)",
                &num_fmt,
            );
        } else if l.indicator_key != "sr" {
            let _ = ws3.write_formula_with_format(
                row,
                18,
                format!("=SUM(F{}:Q{})", excel_row, excel_row).as_str(),
                &num_fmt,
            );
        } else {
            let _ = ws3.write_number_with_format(row, 18, l.annual_realization, &num_fmt);
        }
        if l.indicator_key == "safe_manhours" {
            let _ = ws3.write_string_with_format(row, 19, "Terpantau", &cell_fmt);
        } else if l.indicator_key == "nearmiss" {
            let _ = ws3.write_formula_with_format(
                row,
                19,
                format!(
                    "=IF(S{}<=R{},\"Tercapai (100%)\",\"Perlu Evaluasi\")",
                    excel_row, excel_row
                )
                .as_str(),
                &cell_fmt,
            );
        } else {
            let _ = ws3.write_formula_with_format(
                row,
                19,
                format!(
                    "=IF(S{}<=R{},\"Tercapai (100%)\",\"Tidak Tercapai\")",
                    excel_row, excel_row
                )
                .as_str(),
                &cell_fmt,
            );
        }
        let _ = ws3.write_string_with_format(row, 20, &l.root_cause_analysis, &cell_fmt);
        let _ = ws3.write_string_with_format(row, 21, &l.corrective_action, &cell_fmt);
    }

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| (Status::InternalServerError, Json(ApiError { message: e.to_string() })))?;

    Ok(XlsxFile {
        bytes,
        filename: format!("Sobat_K3_TSP_P2K3_{}.xlsx", period.year),
    })
}

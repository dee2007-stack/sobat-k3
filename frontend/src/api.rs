use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::models::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

async fn parse_resp<T: DeserializeOwned>(resp: gloo_net::http::Response) -> Result<T, String> {
    if resp.ok() {
        resp.json::<T>()
            .await
            .map_err(|e| format!("Gagal parsing respons: {}", e))
    } else {
        let err = resp
            .json::<ApiError>()
            .await
            .map(|e| e.message)
            .unwrap_or_else(|_| format!("HTTP Error {}", resp.status()));
        Err(err)
    }
}

pub async fn get_json<T: DeserializeOwned>(url: &str, token: &str) -> Result<T, String> {
    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {}", e))?;
    parse_resp(resp).await
}

pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    url: &str,
    body: &B,
    token: Option<&str>,
) -> Result<T, String> {
    let mut req = Request::post(url).header("Content-Type", "application/json");
    if let Some(t) = token {
        req = req.header("Authorization", &format!("Bearer {}", t));
    }
    let resp = req
        .json(body)
        .map_err(|e| format!("Serialisasi gagal: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {}", e))?;
    parse_resp(resp).await
}

pub async fn put_json<B: Serialize, T: DeserializeOwned>(
    url: &str,
    body: &B,
    token: &str,
) -> Result<T, String> {
    let resp = Request::put(url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| format!("Serialisasi gagal: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {}", e))?;
    parse_resp(resp).await
}

pub async fn delete_req(url: &str, token: &str) -> Result<(), String> {
    let resp = Request::delete(url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {}", e))?;
    if resp.ok() {
        Ok(())
    } else {
        let err = resp
            .json::<ApiError>()
            .await
            .map(|e| e.message)
            .unwrap_or_else(|_| format!("HTTP Error {}", resp.status()));
        Err(err)
    }
}

pub async fn download_xlsx(period_id: &str, year: i32, token: &str) -> Result<(), String> {
    let url = format!("/api/periods/{}/export/xlsx", period_id);
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {}", e))?;
    if !resp.ok() {
        return Err(format!("Gagal unduh Excel (HTTP {})", resp.status()));
    }
    let bytes = resp
        .binary()
        .await
        .map_err(|e| format!("Gagal membaca file: {}", e))?;
    let uint8 = js_sys::Uint8Array::from(bytes.as_slice());
    let array = js_sys::Array::new();
    array.push(&uint8.buffer());
    let bag = BlobPropertyBag::new();
    bag.set_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &bag)
        .map_err(|_| "Gagal membuat Blob".to_string())?;
    let obj_url =
        Url::create_object_url_with_blob(&blob).map_err(|_| "Gagal membuat URL".to_string())?;
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Ok(el) = doc.create_element("a") {
            if let Ok(a) = el.dyn_into::<HtmlAnchorElement>() {
                a.set_href(&obj_url);
                a.set_download(&format!("Sobat_K3_TSP_P2K3_{}.xlsx", year));
                a.click();
            }
        }
    }
    let _ = Url::revoke_object_url(&obj_url);
    Ok(())
}

// Specific API calls
pub async fn api_login(req: &LoginRequest) -> Result<LoginResponse, String> {
    post_json("/api/auth/login", req, None).await
}

pub async fn api_list_periods(token: &str) -> Result<Vec<DocumentPeriodDto>, String> {
    get_json("/api/periods", token).await
}

pub async fn api_update_period(
    id: &str,
    req: &UpdatePeriodRequest,
    token: &str,
) -> Result<DocumentPeriodDto, String> {
    put_json(&format!("/api/periods/{}", id), req, token).await
}

pub async fn api_sign_period(
    id: &str,
    stage: &str,
    token: &str,
) -> Result<DocumentPeriodDto, String> {
    post_json(
        &format!("/api/periods/{}/sign", id),
        &SignPeriodRequest {
            stage: stage.to_string(),
        },
        Some(token),
    )
    .await
}

pub async fn api_get_dashboard(period_id: &str, token: &str) -> Result<DashboardSummaryDto, String> {
    get_json(&format!("/api/periods/{}/dashboard", period_id), token).await
}

pub async fn api_get_work_hours(
    period_id: &str,
    token: &str,
) -> Result<WorkHoursSummaryDto, String> {
    get_json(&format!("/api/periods/{}/work-hours", period_id), token).await
}

pub async fn api_update_work_hour(
    period_id: &str,
    month: i32,
    req: &UpdateWorkHoursMonthRequest,
    token: &str,
) -> Result<WorkHoursSummaryDto, String> {
    put_json(
        &format!("/api/periods/{}/work-hours/{}", period_id, month),
        req,
        token,
    )
    .await
}

pub async fn api_list_programs(
    period_id: &str,
    token: &str,
) -> Result<Vec<WorkProgramDto>, String> {
    get_json(&format!("/api/periods/{}/programs", period_id), token).await
}

pub async fn api_create_program(
    period_id: &str,
    req: &UpsertWorkProgramRequest,
    token: &str,
) -> Result<WorkProgramDto, String> {
    post_json(
        &format!("/api/periods/{}/programs", period_id),
        req,
        Some(token),
    )
    .await
}

pub async fn api_update_program(
    id: &str,
    req: &UpsertWorkProgramRequest,
    token: &str,
) -> Result<WorkProgramDto, String> {
    put_json(&format!("/api/programs/{}", id), req, token).await
}

pub async fn api_update_program_month(
    id: &str,
    month: i32,
    req: &UpdateProgramMonthRequest,
    token: &str,
) -> Result<WorkProgramDto, String> {
    put_json(&format!("/api/programs/{}/months/{}", id, month), req, token).await
}

pub async fn api_delete_program(id: &str, token: &str) -> Result<(), String> {
    delete_req(&format!("/api/programs/{}", id), token).await
}

pub async fn api_list_lagging(
    period_id: &str,
    token: &str,
) -> Result<Vec<LaggingIndicatorDto>, String> {
    get_json(&format!("/api/periods/{}/lagging", period_id), token).await
}

pub async fn api_update_lagging_meta(
    id: &str,
    req: &UpdateLaggingMetaRequest,
    token: &str,
) -> Result<LaggingIndicatorDto, String> {
    put_json(&format!("/api/lagging/{}", id), req, token).await
}

pub async fn api_update_lagging_month(
    id: &str,
    month: i32,
    req: &UpdateLaggingMonthRequest,
    token: &str,
) -> Result<LaggingIndicatorDto, String> {
    put_json(&format!("/api/lagging/{}/months/{}", id, month), req, token).await
}

pub async fn api_list_users(token: &str) -> Result<Vec<UserDto>, String> {
    get_json("/api/users", token).await
}

pub async fn api_create_user(req: &CreateUserRequest, token: &str) -> Result<UserDto, String> {
    post_json("/api/users", req, Some(token)).await
}

pub async fn api_update_user(
    id: &str,
    req: &UpdateUserRequest,
    token: &str,
) -> Result<UserDto, String> {
    put_json(&format!("/api/users/{}", id), req, token).await
}

pub async fn api_reset_password(
    id: &str,
    req: &ResetPasswordRequest,
    token: &str,
) -> Result<serde_json::Value, String> {
    post_json(&format!("/api/users/{}/reset-password", id), req, Some(token)).await
}

pub async fn api_delete_user(id: &str, token: &str) -> Result<(), String> {
    delete_req(&format!("/api/users/{}", id), token).await
}

pub async fn api_list_audit_logs(token: &str) -> Result<Vec<AuditLogDto>, String> {
    get_json("/api/audit-logs", token).await
}

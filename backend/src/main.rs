mod auth;
mod db;
mod routes;
mod seed;

use dotenvy::dotenv;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Method, Status};
use rocket::{options, routes, Request, Response};
use sqlx::postgres::PgPoolOptions;
use std::env;

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin = request.headers().get_one("Origin").unwrap_or("");
        let allow_origin = match origin {
            "https://www.bpombandung.services" | "https://bpombandung.services" => origin.to_string(),
            _ => env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "https://www.bpombandung.services".to_string()),
        };
        response.set_header(Header::new("Access-Control-Allow-Origin", allow_origin));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, PATCH, DELETE, OPTIONS",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ));
        response.set_header(Header::new("X-Content-Type-Options", "nosniff"));
        response.set_header(Header::new("X-Frame-Options", "SAMEORIGIN"));
        if request.method() == Method::Options {
            response.set_status(Status::NoContent);
        }
    }
}

#[options("/<_..>")]
fn all_options() -> Status {
    Status::NoContent
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dee@localhost:5432/sobat_k3".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Gagal terhubung ke PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Gagal menjalankan migrasi PostgreSQL");

    seed::run_seed(&pool)
        .await
        .expect("Gagal menjalankan seed awal");

    let port: u16 = env::var("ROCKET_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);

    let address = env::var("ROCKET_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

    use rocket::data::ToByteUnit;
    let figment = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", address))
        .merge(("limits", rocket::data::Limits::default().limit("file", 10.mebibytes())));

    let _rocket = rocket::custom(figment)
        .attach(Cors)
        .manage(pool)
        .mount(
            "/",
            routes![
                all_options,
                routes::auth::login,
                routes::auth::me,
                routes::users::list_users,
                routes::users::create_user,
                routes::users::update_user,
                routes::users::reset_password,
                routes::users::delete_user,
                routes::users::list_audit_logs,
                routes::periods::list_periods,
                routes::periods::update_period,
                routes::periods::sign_period,
                routes::work_hours::get_work_hours,
                routes::work_hours::update_work_hours_month,
                routes::programs::list_programs,
                routes::programs::create_program,
                routes::programs::update_program,
                routes::programs::update_program_month,
                routes::programs::delete_program,
                routes::programs::upload_evidence,
                routes::programs::download_evidence,
                routes::lagging::list_lagging,
                routes::lagging::update_lagging_meta,
                routes::lagging::update_lagging_month,
                routes::export::get_dashboard,
                routes::export::export_xlsx,
            ],
        )
        .launch()
        .await?;

    Ok(())
}

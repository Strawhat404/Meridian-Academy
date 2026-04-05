#[macro_use]
extern crate rocket;

mod middleware;
mod models;
mod routes;

use rocket::fairing::AdHoc;
use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use bcrypt;

pub type DbPool = sqlx::MySqlPool;

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    run_migrations(&pool).await;

    // Spawn nightly reconciliation report generator
    {
        let sched_pool = pool.clone();
        tokio::spawn(async move {
            loop {
                routes::payments::generate_reconciliation_report(&sched_pool).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(24 * 60 * 60)).await;
            }
        });
    }

    // Restrict CORS to explicit local origins only
    let frontend_url = env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let allowed_origins = AllowedOrigins::some_exact(&[
        frontend_url,
        "http://localhost:8080".to_string(),
        "http://127.0.0.1:8080".to_string(),
    ]);
    let cors = CorsOptions {
        allowed_origins,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("CORS configuration error");

    rocket::build()
        .manage(pool)
        .attach(cors)
        .attach(AdHoc::config::<models::AppConfig>())
        .mount("/", routes::health_routes())
        .mount("/api/auth", routes::auth_routes())
        .mount("/api/users", routes::user_routes())
        .mount("/api/submissions", routes::submission_routes())
        .mount("/api/orders", routes::order_routes())
        .mount("/api/reviews", routes::review_routes())
        .mount("/api/cases", routes::case_routes())
        .mount("/api/payments", routes::payment_routes())
        .mount("/api/admin", routes::admin_routes())
        .mount("/api/content", routes::content_routes())
}

/// Run SQL migrations — fail-fast on errors (panics on critical failures).
async fn run_migrations(pool: &DbPool) {
    let migrations = include_str!("migrations/001_initial.sql");
    for statement in migrations.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            match sqlx::query(trimmed).execute(pool).await {
                Ok(_) => {}
                Err(e) => {
                    let err_str = e.to_string();
                    // "already exists" is expected on re-runs — safe to skip
                    if err_str.contains("already exists") || err_str.contains("Duplicate") {
                        log::debug!("Migration skip (idempotent): {}", err_str);
                    } else {
                        panic!("FATAL: Migration failed — {}", err_str);
                    }
                }
            }
        }
    }
    log::info!("Database migrations completed");

    let seed = include_str!("migrations/002_seed.sql");
    for statement in seed.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            match sqlx::query(trimmed).execute(pool).await {
                Ok(_) => {}
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Duplicate") {
                        log::debug!("Seed skip (idempotent): {}", err_str);
                    } else {
                        panic!("FATAL: Seed data failed — {}", err_str);
                    }
                }
            }
        }
    }
    log::info!("Seed data applied");

    // If ADMIN_PASSWORD is set, re-hash and apply to all seeded accounts
    if let Ok(admin_pass) = std::env::var("ADMIN_PASSWORD") {
        match bcrypt::hash(&admin_pass, bcrypt::DEFAULT_COST) {
            Ok(hash) => {
                let _ = sqlx::query(
                    "UPDATE users SET password_hash = ? WHERE username IN ('admin', 'moderator', 'host', 'clerk', 'member')"
                )
                .bind(&hash)
                .execute(pool)
                .await;
                log::info!("Seeded user credentials updated from ADMIN_PASSWORD");
            }
            Err(e) => log::warn!("Failed to hash ADMIN_PASSWORD: {}", e),
        }
    }
}

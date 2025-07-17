use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod database;
mod models;
mod auth;
mod api;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "glad_web=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    database::init_db()
        .await
        .expect("Failed to initialize database");

    // Create router
    let app = Router::new()
        // API routes
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/logout", post(api::auth::logout))
        .route("/api/auth/signup", post(api::auth::signup))
        .route("/api/auth/me", get(api::auth::me))
        .route("/api/auth/reset-password", post(api::auth::reset_password))
        .route("/api/auth/reset-password-confirm", post(api::auth::reset_password_confirm))
        .route("/api/auth/settings", post(api::auth::update_settings))
        .route("/api/cohorts", get(api::find::get_cohorts))
        .route("/api/find-controls", post(api::find::submit_find_controls))
        .route("/api/queries", get(api::find::get_user_queries))
        .route("/api/queries/{id}", get(api::find::get_query_details))
        .route("/api/pca-data", get(api::explore::get_pca_data))
        // Static file serving for frontend
        .fallback_service(ServeDir::new("frontend/build"))
        // Middleware
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(auth::middleware::auth_middleware));

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");
    
    tracing::info!("Server listening on http://127.0.0.1:3000");
    
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

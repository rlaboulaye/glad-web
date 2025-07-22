use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod database;
mod models;
mod visualization;

/// Number of top communities to use for cache warming
const CACHE_WARMING_COMMUNITIES_NUM: usize = 16;

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

    // Warm visualization cache
    tracing::info!("Warming visualization cache...");
    tokio::spawn(async {
        // First warm the basic cache (communities, groups, etc.)
        match api::explore::get_ibd_communities(axum::extract::Query(
            api::explore::CommunitiesQuery { limit: None },
        ))
        .await
        {
            Ok(_) => tracing::info!("Basic visualization cache warmed successfully"),
            Err(e) => tracing::error!("Failed to warm basic visualization cache: {:?}", e),
        }

        // Then warm the IBD matrix computation for top k communities (default view)
        tracing::info!(
            "Warming IBD matrix cache for top {} communities...",
            CACHE_WARMING_COMMUNITIES_NUM
        );

        // Get the top k community labels
        match api::explore::get_ibd_groups(axum::extract::Query(api::explore::IbdGroupsQuery {
            grouping: "ibd_community".to_string(),
            min_size: Some(30),
        }))
        .await
        {
            Ok(groups_response) => {
                // Parse the JSON response
                if let Ok(groups_data) =
                    serde_json::from_value::<serde_json::Value>(groups_response.0.clone())
                {
                    if let Some(groups_array) = groups_data.get("groups").and_then(|v| v.as_array())
                    {
                        let top_k_labels: Vec<String> = groups_array
                            .iter()
                            .filter_map(|g| {
                                let label = g.get("label")?.as_str()?;
                                if label != "Unknown" {
                                    Some(label.to_string())
                                } else {
                                    None
                                }
                            })
                            .take(CACHE_WARMING_COMMUNITIES_NUM)
                            .collect();

                        if !top_k_labels.is_empty() {
                            let matrix_request = api::explore::IbdMatrixQuery {
                                grouping: "ibd_community".to_string(),
                                selected_groups: top_k_labels.clone(),
                            };

                            match api::explore::compute_ibd_matrix(axum::extract::Json(
                                matrix_request,
                            ))
                            .await
                            {
                                Ok(_) => tracing::info!(
                                    "IBD matrix cache warmed for {} communities",
                                    top_k_labels.len()
                                ),
                                Err(e) => {
                                    tracing::error!("Failed to warm IBD matrix cache: {:?}", e)
                                }
                            }
                        } else {
                            tracing::warn!("No valid communities found for cache warming");
                        }
                    } else {
                        tracing::error!("Invalid groups response format for cache warming");
                    }
                } else {
                    tracing::error!("Failed to parse groups response for cache warming");
                }
            }
            Err(e) => tracing::error!("Failed to get communities for cache warming: {:?}", e),
        }
    });

    // Create router
    let app = Router::new()
        // API routes
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/logout", post(api::auth::logout))
        .route("/api/auth/signup", post(api::auth::signup))
        .route("/api/auth/me", get(api::auth::me))
        .route("/api/auth/reset-password", post(api::auth::reset_password))
        .route(
            "/api/auth/reset-password-confirm",
            post(api::auth::reset_password_confirm),
        )
        .route("/api/auth/settings", post(api::auth::update_settings))
        .route("/api/cohorts", get(api::find::get_cohorts))
        .route("/api/find-controls", post(api::find::submit_find_controls))
        .route("/api/queries", get(api::find::get_user_queries))
        .route("/api/queries/{id}", get(api::find::get_query_details))
        .route("/api/pca-data", get(api::explore::get_pca_data))
        .route(
            "/api/ibd-communities",
            get(api::explore::get_ibd_communities),
        )
        .route("/api/ibd-groups", get(api::explore::get_ibd_groups))
        .route("/api/ibd-matrix", post(api::explore::compute_ibd_matrix))
        .route("/api/ibd-matrix-asymmetric", post(api::explore::compute_asymmetric_ibd_matrix))
        // Static file serving for frontend
        .fallback_service(ServeDir::new("frontend/build"))
        // Middleware
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
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

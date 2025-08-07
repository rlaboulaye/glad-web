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

/// Number of top groups to use for cache warming (matches frontend MAX_DEFAULT_SELECTED_GROUPS)
const CACHE_WARMING_TOP_GROUPS: usize = 12;

/// Notification check interval in seconds
const NOTIFICATION_CHECK_INTERVAL_SECONDS: u64 = 60;

/// Warm cache for a specific field combination (single field or comma-separated fields)
async fn warm_field_cache(
    grouping: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    // Get groups for this field combination
    let groups_response =
        api::explore::get_ibd_groups(axum::extract::Query(api::explore::IbdGroupsQuery {
            grouping: grouping.to_string(),
            min_size: None,
        }))
        .await
        .map_err(|e| format!("Failed to get groups: {:?}", e))?;

    // Parse the JSON response
    let groups_data = serde_json::from_value::<serde_json::Value>(groups_response.0)?;

    if let Some(groups_array) = groups_data.get("groups").and_then(|v| v.as_array()) {
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
            .take(CACHE_WARMING_TOP_GROUPS)
            .collect();

        if !top_k_labels.is_empty() {
            let matrix_request = api::explore::IbdMatrixQuery {
                grouping: grouping.to_string(),
                selected_groups: top_k_labels.clone(),
            };

            api::explore::compute_ibd_matrix(axum::extract::Json(matrix_request))
                .await
                .map_err(|e| format!("Failed to compute matrix: {:?}", e))?;
            Ok(top_k_labels.len())
        } else {
            Ok(0)
        }
    } else {
        Err("Invalid groups response format".into())
    }
}

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

        // Enhanced cache warming: individual metadata fields and pairs
        tracing::info!("Starting enhanced cache warming for metadata fields...");

        // Warm cache for individual metadata fields
        tracing::info!("Warming cache for individual metadata fields...");
        for field in visualization::CANONICAL_FIELD_ORDER {
            match warm_field_cache(field).await {
                Ok(count) => tracing::info!("Warmed {} top groups for field: {}", count, field),
                Err(e) => tracing::error!("Failed to warm cache for field '{}': {:?}", field, e),
            }
        }

        // Warm cache for pairs of metadata fields
        tracing::info!("Warming cache for pairs of metadata fields...");
        for (i, field1) in visualization::CANONICAL_FIELD_ORDER.iter().enumerate() {
            for field2 in visualization::CANONICAL_FIELD_ORDER.iter().skip(i + 1) {
                let field_combination = format!("{},{}", field1, field2);
                match warm_field_cache(&field_combination).await {
                    Ok(count) => tracing::info!(
                        "Warmed {} top groups for fields: {}",
                        count,
                        field_combination
                    ),
                    Err(e) => tracing::error!(
                        "Failed to warm cache for fields '{}': {:?}",
                        field_combination,
                        e
                    ),
                }
            }
        }

        tracing::info!("Enhanced cache warming completed");
    });

    // Start notification monitoring task
    tracing::info!("Starting notification monitoring task...");
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
            NOTIFICATION_CHECK_INTERVAL_SECONDS,
        ));

        loop {
            interval.tick().await;

            match models::Notification::process_pending_notifications().await {
                Ok(count) if count > 0 => {
                    tracing::info!("Processed {} pending notifications", count);
                }
                Ok(_) => {
                    // No notifications to process - keep quiet to avoid spam
                }
                Err(e) => {
                    tracing::error!("Failed to process pending notifications: {}", e);
                }
            }
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
        .route(
            "/api/ibd-matrix-asymmetric",
            post(api::explore::compute_asymmetric_ibd_matrix),
        )
        .route("/api/citations", get(api::publication::get_citations))
        // Notification routes
        .route(
            "/api/notifications",
            get(api::notifications::get_notifications),
        )
        .route(
            "/api/notifications/unread-count",
            get(api::notifications::get_unread_count),
        )
        .route(
            "/api/notifications/mark-read",
            post(api::notifications::mark_as_read),
        )
        .route(
            "/api/notifications/mark-all-read",
            post(api::notifications::mark_all_as_read),
        )
        .route(
            "/api/notifications/process-pending",
            post(api::notifications::process_pending_notifications),
        )
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

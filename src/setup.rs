use axum::Router;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

use crate::app::{shell, App};

/// # Panics
///
/// Will panic if anything is badly setup from database, or web server
pub async fn init_app(configuration_path: Option<&str>) {
    tracing_subscriber::fmt()
        .with_level(true)
        .with_max_level(tracing::Level::INFO)
        .init();
    // Init the pool into static
    crate::database::init_db()
        .await
        .expect("problem during initialization of the database");

    // Get leptos configuration
    let conf = get_configuration(configuration_path).unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    let leptos_options = conf.leptos_options;

    let serve_dir = tower_http::services::ServeDir::new(&*leptos_options.site_root)
        .append_index_html_on_directories(false);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback_service(serve_dir)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                )
                .on_failure(
                    tower_http::trace::DefaultOnFailure::new().level(tracing::Level::ERROR),
                ),
        )
        .layer(axum::middleware::from_fn(crate::auth::auth_middleware))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

use crate::app_state::AppState;
use crate::config::Config;
use crate::handlers::*;
use crate::middleware::*;
use anyhow::{Context, Result};
use axum::middleware::from_fn;
use axum::{Router, http::HeaderValue, routing::get};
use moka::future::Cache;
use resource_io::{ResourceLoader, ResourceLoaderConfig};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub mod app_state;
pub mod config;
pub mod handlers;
pub mod layer_definition;
pub mod logging;
pub mod middleware;
pub mod s2_utils;
pub mod tiles3d;
pub mod utils;
pub mod viewer;

#[tokio::main]
async fn main() -> Result<()> {
    // Slurp from .env
    dotenvy::dotenv().ok();

    // Load config
    let config = Config::load();

    // Setup logging
    logging::setup_logging(&config.log_level, config.pretty_log);

    // Using moka here to get ttl
    let layer_definition_cache = Cache::builder()
        .time_to_live(config.layer_definition_ttl)
        .max_capacity(1_024) // TODO: from config
        .build();

    let resource_loader_config = ResourceLoaderConfig {
        block_cache_bytes: config.block_cache_size.as_u64(),
        ..Default::default()
    };

    let app_state = AppState {
        config: config.clone(),
        resource_loader: ResourceLoader::new(resource_loader_config).await,
        layer_definition_cache: Arc::new(layer_definition_cache),
    };

    let app = Router::new()
        // These routes expire quickly, in case the config is updated
        .route("/{id}", get(get_root_tileset))
        .route_layer(from_fn(cache_short))
        // These routes are immutable, we depend on the id/hash of config to bust cache
        .route("/{id}/{hash}/tileset", get(get_root_tileset_top_node))
        .route(
            "/{id}/{hash}/tileset/{face}/{level}/{col}/{row}",
            get(get_child_tileset),
        )
        .route(
            "/{id}/{hash}/content/{token}/top",
            get(get_content_toplevel),
        )
        .route(
            "/{id}/{hash}/content/{token}/{*rest}",
            get(get_content_payload),
        )
        .route(
            "/{id}/{hash}/bg_content/{*rest}",
            get(get_base_globe_terrain_payload),
        )
        .route_layer(from_fn(cache_forever))
        .fallback(viewer::static_handler)
        .with_state(app_state.clone());

    let app = match config.cors_origin.as_deref() {
        None => app,
        Some("*") => app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        ),
        Some(origin) => app.layer(
            CorsLayer::new()
                .allow_origin(origin.parse::<HeaderValue>().context("Bad CORS origin")?)
                .allow_methods(Any)
                .allow_headers(Any),
        ),
    };

    info!("🚀 Listening on {}", app_state.config.listen_addr);

    let listener = TcpListener::bind(config.listen_addr).await?;
    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");
    tracing::info!("Shutting down...");
}

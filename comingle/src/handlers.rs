use crate::app_state::AppState;
use crate::tiles3d;
use crate::utils::*;
use anyhow::Result;
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::IntoResponse;
use axum::{Json, extract::Path, extract::State, http::StatusCode};
use iri_string::types::{UriAbsoluteStr, UriReferenceStr};
use serde::Deserialize;

pub async fn get_root_tileset(
    State(app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<tiles3d::Tileset>, StatusCode> {
    let layer_def = app_state
        .get_layer_definition(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let tileset = layer_def.root_tileset();
    Ok(Json(tileset))
}

#[derive(Deserialize)]
pub struct GetRootTilesetTopNodePaths {
    pub id: String,
}

pub async fn get_root_tileset_top_node(
    State(app_state): State<AppState>,
    Path(paths): Path<GetRootTilesetTopNodePaths>,
) -> Result<Json<tiles3d::Tileset>, StatusCode> {
    let layer_def = app_state
        .get_layer_definition(&paths.id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let tileset = layer_def.synthesize_s2_root();
    Ok(Json(tileset))
}

#[derive(Deserialize)]
pub struct GetChildTilesetPaths {
    pub id: String,
    pub face: u8,
    pub level: i32,
    pub col: i32,
    pub row: i32,
}

pub async fn get_child_tileset(
    State(app_state): State<AppState>,
    Path(paths): Path<GetChildTilesetPaths>,
) -> Result<Json<tiles3d::Tileset>, StatusCode> {
    // TODO: If level is >= content level, reach into the tileset and get the tileset, walk it and find the
    // child for face/level/col/row
    // TODO: Use face/level/col/row to figure out which content to reach into

    // When we repack tileset json from within a 3tz, we can strip out the tileset metadata
    // and replace it so we have consistent.

    // TODO: If the level is less than the content level, we synthesize a tile.
    // If it's >=, we want to get the tile from the appropriate tileset.
    // For now, we synthesize all tilesets

    let layer_def = app_state
        .get_layer_definition(&paths.id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let tileset = layer_def.synthesize_tileset(paths.face, paths.level, paths.col, paths.row);
    Ok(Json(tileset))
}

#[derive(Deserialize)]
pub struct GetContentToplevelPaths {
    pub id: String,
    pub token: String,
}

pub async fn get_content_toplevel(
    State(app_state): State<AppState>,
    Path(paths): Path<GetContentToplevelPaths>,
) -> Result<Json<tiles3d::Tileset>, StatusCode> {
    let layer_def = app_state
        .get_layer_definition(&paths.id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut content_root = layer_def.resolve_content_uri_template(&paths.token);
    if content_root.ends_with(".3tz") {
        content_root.push_str("/tileset.json");
    }
    let uri = UriAbsoluteStr::new(&content_root).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: We should probably cache this

    let tileset = get_content_root_tileset(
        app_state.resource_loader.clone(),
        uri,
        layer_def.source_s2_content_level,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tileset))
}

#[derive(Deserialize)]
pub struct GetContentPayloadPaths {
    pub id: String,
    pub token: String,
    pub rest: String,
}

// ..this is where content transform pipeline would run
// ..if there is no transform, we can just reframe deflated compressed entry from 3tz, or zstd
// note that means we probably want some helper on resource loader to get the compressed content and method
// TODO: If level < content level, read from one of the bg terrain files
pub async fn get_content_payload(
    State(app_state): State<AppState>,
    Path(paths): Path<GetContentPayloadPaths>,
) -> Result<impl IntoResponse, StatusCode> {
    let layer_def = app_state
        .get_layer_definition(&paths.id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut content_root = layer_def.resolve_content_uri_template(&paths.token);
    if content_root.ends_with(".3tz") {
        content_root.push_str("/tileset.json");
    }
    let root = UriAbsoluteStr::new(&content_root).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let relative = UriReferenceStr::new(&paths.rest).map_err(|_| StatusCode::BAD_REQUEST)?;
    let resolved = relative.resolve_against(root).to_string();
    let uri = UriAbsoluteStr::new(&resolved).map_err(|_| StatusCode::BAD_REQUEST)?;
    let bytes = app_state
        .resource_loader
        .read_async(uri)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?; // TODO: distinguish 404 vs 500

    // TODO: If the payload is a tileset, we need to (temporarily, until CesiumJS is fixed), strip the tileset metadata and schema.
    // Or maybe just add the schema to the toplevel schema for terrain in the tileset. That's probably best.

    let content_type = sniff_content_type(&bytes);
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));

    Ok((headers, bytes))
}

#[derive(Deserialize)]
pub struct GetBaseGlobeTerrainPayloadPaths {
    pub id: String,
    pub rest: String,
}

pub async fn get_base_globe_terrain_payload(
    State(app_state): State<AppState>,
    Path(paths): Path<GetBaseGlobeTerrainPayloadPaths>,
) -> Result<impl IntoResponse, StatusCode> {
    let layer_def = app_state
        .get_layer_definition(&paths.id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if layer_def.base_globe_terrain_uri.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let glb_uri = format!(
        "{}/{}",
        layer_def.base_globe_terrain_uri.as_ref().unwrap(),
        paths.rest
    );
    let uri = UriAbsoluteStr::new(&glb_uri).map_err(|_| StatusCode::BAD_REQUEST)?;
    let bytes = app_state
        .resource_loader
        .read_async(uri)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?; // TODO: distinguish 404 vs 500

    let content_type = sniff_content_type(&bytes);
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));

    Ok((headers, bytes))
}

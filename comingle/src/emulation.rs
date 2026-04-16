use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Json, response::Redirect};
use iri_string::types::{IriAbsoluteStr, IriRelativeStr};
use serde::{Deserialize, Serialize};

// A dummy JWT that expires in 2040, by that time, we better not be using this!
const DUMMY_JWT: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjIyMTgxODU2MDB9.NgAvss9CK3RT09DsCzXHvE_uMbqp-0UJjcotIJwc0U8";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthenticationMode {
    CesiumIon,
    Saml,
    SingleUser,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataStoreType {
    FileSystem,
    S3,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDataResponse {
    pub authentication_mode: AuthenticationMode,
    pub data_store_type: DataStoreType,
    pub attribution: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetType {
    #[serde(rename(serialize = "3DTILES"))]
    Tiles3d,
    Gltf,
    Imagery,
    Terrain,
    Kml,
    Czml,
    Geojson,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetStatus {
    AwaitingFiles,
    NotStarted,
    InProgress,
    Complete,
    DataError,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAssetsResponse {
    pub items: Vec<Asset>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    pub r#type: AssetType,
    pub bytes: i64,
    pub date_added: String, // RFC-3339
    pub status: AssetStatus,
    pub percent_complete: i32,
    pub exportable: bool,
    pub creator_id: i64,
    pub creator_username: String,
}

impl From<&crate::layer_definition::LayerDefinition> for Asset {
    fn from(layer: &crate::layer_definition::LayerDefinition) -> Self {
        Asset {
            id: layer.asset_id,
            name: layer.id.clone(),
            description: Some(layer.id.clone()),
            attribution: None,
            r#type: AssetType::Tiles3d,
            bytes: 1_000_000,
            date_added: "2026-04-01T00:00:00Z".to_string(),
            status: AssetStatus::Complete,
            percent_complete: 100,
            exportable: false,
            creator_id: 1,
            creator_username: "Admin".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetEndpointAttribution {
    pub html: String,
    pub collapsible: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetEndpoint {
    pub r#type: AssetType,
    pub url: String,
    pub access_token: String,
    pub attributions: Vec<AssetEndpointAttribution>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct External3dtilesAssetOptions {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExternalType {
    Bing,
    #[serde(rename(serialize = "3DTILES"))]
    Tiles3d,
    Google2dMaps,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAssetEndpoint {
    pub r#type: AssetType,
    pub external_type: ExternalType,
    pub attributions: Vec<AssetEndpointAttribution>,
    pub options: External3dtilesAssetOptions,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum GetAssetEndpointResponse {
    AssetEndpoint(AssetEndpoint),
    ExternalAssetEndpoint(ExternalAssetEndpoint),
}

pub async fn app_data() -> Json<AppDataResponse> {
    Json(AppDataResponse {
        authentication_mode: AuthenticationMode::SingleUser,
        data_store_type: DataStoreType::FileSystem,
        attribution: "".to_string(),
    })
}

#[derive(Deserialize)]
pub struct AuthRequest {
    redirect_uri: String,
    state: String,
}

pub async fn oauth(Query(auth): Query<AuthRequest>) -> Redirect {
    let target = format!("{}?code=EMPTY_CODE&state={}", auth.redirect_uri, auth.state);
    Redirect::temporary(&target)
}

#[derive(Serialize)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

pub async fn oauth_token() -> Json<TokenResponse> {
    Json(TokenResponse {
        access_token: DUMMY_JWT.to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 315_360_000, // 10 years in seconds
    })
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IonToken {
    pub id: String,
    pub name: String,
    pub token: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub assets: Vec<i64>,
    pub is_default: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListTokensResponse {
    pub items: Vec<IonToken>,
}
pub async fn list_tokens() -> Json<ListTokensResponse> {
    Json(ListTokensResponse {
        items: vec![IonToken {
            id: "1".to_string(),
            name: "Default".to_string(),
            token: DUMMY_JWT.to_string(),
            description: Some("Default token".to_string()),
            scopes: vec!["assets:read".to_string(), "profile:read".to_string()],
            assets: vec![], // Empty means all assets
            is_default: true,
        }],
    })
}

#[derive(Serialize)]
pub struct Me {
    id: i64,
    username: String,
    email: String,
    storage_quota: i64,
}

pub async fn me() -> Json<Me> {
    Json(Me {
        id: 1,
        username: "User".to_string(),
        email: "user@example.com".to_string(),
        storage_quota: 100_000_000,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultsResponse {
    pub default_terrain_asset_id: i64,
    pub default_imagery_asset_id: i64,
}

pub async fn get_defaults() -> Json<DefaultsResponse> {
    Json(DefaultsResponse {
        default_terrain_asset_id: 0,
        default_imagery_asset_id: 0,
    })
}

pub async fn list_assets(
    State(app_state): State<AppState>,
) -> Result<Json<ListAssetsResponse>, StatusCode> {
    let layers = app_state
        .get_layer_definitions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut items: Vec<Asset> = layers.iter().map(|layer| Asset::from(&**layer)).collect();
    items.sort_by_key(|asset| asset.id);
    Ok(Json(ListAssetsResponse { items }))
}

pub async fn get_asset(
    State(app_state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Asset>, StatusCode> {
    let layers = app_state
        .get_layer_definitions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let layer = layers
        .iter()
        .find(|l| l.asset_id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(Asset::from(&**layer)))
}

pub async fn get_asset_endpoint(
    State(app_state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<GetAssetEndpointResponse>, StatusCode> {
    let layers = app_state
        .get_layer_definitions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let layer = layers
        .iter()
        .find(|l| l.asset_id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let base_uri = IriAbsoluteStr::new(&app_state.config.base_url)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let layer_uri =
        IriRelativeStr::new(&layer.id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let url = layer_uri.resolve_against(base_uri);

    Ok(Json(GetAssetEndpointResponse::ExternalAssetEndpoint(
        ExternalAssetEndpoint {
            r#type: AssetType::Tiles3d,
            external_type: ExternalType::Tiles3d,
            attributions: vec![],
            options: External3dtilesAssetOptions {
                url: url.to_string(),
            },
        },
    )))
}

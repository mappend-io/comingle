use serde::{Deserialize, Serialize};

// https://docs.ogc.org/cs/22-025r4/22-025r4.html#toc117
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootProperty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

impl RootProperty {
    pub fn get_extension<T: serde::de::DeserializeOwned>(&self, name: &str) -> Option<T> {
        let extensions = self.extensions.as_ref()?;
        let value = extensions.get(name)?;
        serde_json::from_value(value.clone()).ok()
    }
}

// https://docs.ogc.org/cs/22-025r4/22-025r4.html#toc134
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tileset {
    pub asset: Asset,
    pub root: Tile,
    pub geometric_error: f64,
    // metadata
    // groups
    // statistics
    // schema OR schema_uri
    // properties (deprecated)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extensions_used: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extensions_required: Vec<String>,
    #[serde(flatten)]
    pub root_property: RootProperty,
}

// https://docs.ogc.org/cs/22-025r4/22-025r4.html#toc94
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tileset_version: Option<String>,

    #[serde(flatten)]
    pub root_property: RootProperty,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            version: "1.1".to_string(),
            tileset_version: None,
            root_property: RootProperty::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RefineMode {
    Replace,
    Add,
}

// https://docs.ogc.org/cs/22-025r4/22-025r4.html#toc132
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tile {
    pub bounding_volume: BoundingVolume,
    // Option viewer_request_volume
    pub geometric_error: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refine: Option<RefineMode>,
    // Option transform

    // TODO: Option content OR contents
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,

    // Option metadata
    // Option implicit_tiling
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Tile>,

    #[serde(flatten)]
    pub root_property: RootProperty,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingVolume {
    #[serde(flatten)]
    pub shape: BoundingVolumeShape,

    #[serde(flatten)]
    pub root_property: RootProperty,
}

// https://docs.ogc.org/cs/22-025r4/22-025r4.html#reference-content
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_volume: Option<BoundingVolume>,
    pub uri: String,
    //pub metadata: Option<Metadata>,
    //group
    #[serde(flatten)]
    pub root_property: RootProperty,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BoundingVolumeShape {
    Box([f64; 12]),
    Region([f64; 6]),
    Sphere([f64; 4]),
}

impl BoundingVolume {
    pub fn from_lat_lng_elev_degrees(
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        min_height: f64,
        max_height: f64,
    ) -> Self {
        Self {
            shape: BoundingVolumeShape::Region([
                west.to_radians(),
                south.to_radians(),
                east.to_radians(),
                north.to_radians(),
                min_height,
                max_height,
            ]),
            root_property: RootProperty::default(),
        }
    }
}

use crate::tiles3d;
use anyhow::Result;
use iri_string::types::{UriAbsoluteStr, UriAbsoluteString, UriReferenceStr};
use resource_io::ResourceLoader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileExtMaxarGrid {
    bounding_box: [f64; 6],
    face: i32,
    index: [i32; 2],
    level: i32,
}

fn uri_relative_to_base_dir(resolved: &str, base_uri: &UriAbsoluteStr) -> String {
    let base_str = base_uri.as_str();
    let base_dir = base_str
        .rfind('/')
        .map(|i| &base_str[..=i])
        .unwrap_or(base_str);

    let base_segments: Vec<&str> = base_dir.split('/').collect();
    let resolved_segments: Vec<&str> = resolved.split('/').collect();

    // Find where base_dir and resolved diverge
    let diverge_at = base_segments
        .iter()
        .zip(resolved_segments.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // The relative path is everything in resolved from the divergence point
    resolved_segments[diverge_at..].join("/")
}

fn rewrite_tile_uris_relative_to_root(
    tile: &mut tiles3d::Tile,
    tile_base_uri: &UriAbsoluteStr,
    root_tileset_uri: &UriAbsoluteStr,
) -> Result<()> {
    if let Some(content) = &mut tile.content {
        // Resolve relative to where the tile is actually located
        let relative = UriReferenceStr::new(&content.uri)?;
        let resolved = relative.resolve_against(tile_base_uri).to_string();

        // Make it relative to the root tileset's base directory
        content.uri = uri_relative_to_base_dir(&resolved, root_tileset_uri);
    }

    for child in &mut tile.children {
        rewrite_tile_uris_relative_to_root(child, tile_base_uri, root_tileset_uri)?;
    }

    Ok(())
}

pub async fn get_content_root_tileset(
    resource_loader: ResourceLoader,
    root_tileset_uri: &UriAbsoluteStr,
    content_level: i32,
) -> Result<tiles3d::Tileset> {
    let bytes = resource_loader.read_async(root_tileset_uri).await?;
    let root_tileset = serde_json::from_slice::<tiles3d::Tileset>(&bytes)?;

    // We are seeking out a single tiles3d::Tile that has the MAXAR_grid extension on it with a level value of content_level.
    // We will then slurp out that one tile and make it the root of a new tileset and return it.
    // We probably have to recurse into external tilesets.

    let (mut tile, tile_base_uri) = find_tile_at_level(
        resource_loader.clone(),
        root_tileset_uri,
        &root_tileset.root,
        content_level,
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("No tile found at level {content_level}"))?;

    // TODO: Go through any URIs in root and rewrite them based on where we slurped this from.
    // Since this is being mapped to /<tileset_id>/<some_hash>/content/top, if we slurped it from xyz.3tz/4/4/12/34.json, it might reference
    // children as ../../8/123/456.json. But that won't work if we serve this from top.
    // So we somehow need to return the uri we pulled the data from in find_tile_at_level, and turn a ../../8/123/456.json to 8/123/456.json.
    rewrite_tile_uris_relative_to_root(&mut tile, &tile_base_uri, root_tileset_uri)?;

    let tileset = tiles3d::Tileset {
        root: tile,
        ..root_tileset
    };

    Ok(tileset)
}

#[async_recursion::async_recursion]
async fn find_tile_at_level(
    resource_loader: ResourceLoader,
    base_uri: &UriAbsoluteStr,
    tile: &tiles3d::Tile,
    content_level: i32,
) -> Result<Option<(tiles3d::Tile, UriAbsoluteString)>> {
    // Check if this tile has the MAXAR_grid extension at the right level
    if let Some(ext) = tile
        .root_property
        .get_extension::<TileExtMaxarGrid>("MAXAR_grid")
        && ext.level == content_level
    {
        return Ok(Some((tile.clone(), base_uri.to_owned())));
    }

    if let Some(content) = &tile.content
        && content.uri.ends_with(".json")
    {
        let relative = UriReferenceStr::new(&content.uri)?;
        let resolved_external_uri = relative.resolve_against(base_uri).to_string();
        let external_uri = UriAbsoluteStr::new(&resolved_external_uri)?;
        let bytes = resource_loader.read_async(external_uri).await?;
        let external_tileset = serde_json::from_slice::<tiles3d::Tileset>(&bytes)?;
        if let Some(found) = find_tile_at_level(
            resource_loader.clone(),
            external_uri,
            &external_tileset.root,
            content_level,
        )
        .await?
        {
            return Ok(Some(found));
        }
    }

    for child in &tile.children {
        if let Some(found) =
            find_tile_at_level(resource_loader.clone(), base_uri, child, content_level).await?
        {
            return Ok(Some(found));
        }
    }

    Ok(None)
}

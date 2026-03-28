use crate::config::Config;
use crate::layer_definition::LayerDefinition;
use anyhow::{Context, Result, bail};
use iri_string::types::{UriAbsoluteStr, UriReferenceStr};
use moka::future::Cache;
use resource_io::ResourceLoader;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub resource_loader: ResourceLoader,
    pub layer_definition_cache: Arc<Cache<String, Arc<LayerDefinition>>>,
}

fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl AppState {
    pub async fn get_layer_definition(&self, id: &str) -> Result<Arc<LayerDefinition>> {
        if !is_valid_identifier(id) {
            bail!("Invalid id, must be a valid identifier");
        }
        let config = self.config.clone();
        let id_owned = id.to_string();
        let resource_loader = self.resource_loader.clone();
        self.layer_definition_cache
            .try_get_with(id_owned.clone(), async move {
                let root = UriAbsoluteStr::new(&config.layer_config_uri)?;
                let file_name = format!("{id}.json");
                let relative = UriReferenceStr::new(&file_name)?;
                let resolved = relative.resolve_against(root).to_string();
                let absolute = UriAbsoluteStr::new(&resolved)?;
                let bytes = resource_loader.read_async(absolute).await?;
                let mut def = serde_json::from_slice::<LayerDefinition>(&bytes)?;
                def.id = id_owned;
                Ok::<_, anyhow::Error>(Arc::new(def))
            })
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .context("Could not fetch layer definition")
    }
}

use serde::Deserialize;
use lazy_static::lazy_static;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub struct SystemConfig {
    pub rpc_timeout_ms: u64,
    pub server_port: u16,
    pub dashboard_port: u16,
    pub metrics_endpoint: String,
    
    // New fields we added to remove hardcoding
    #[serde(default = "default_storage_path")]
    pub storage_path: String,
    #[serde(default = "default_agent_id")]
    pub default_agent_id: String,
    #[serde(default = "default_admin_id")]
    pub default_admin_id: String,
    #[serde(default = "default_meta_workspace")]
    pub meta_workspace: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComputeConfig {
    pub max_inference_threads: usize,
    pub max_execution_workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelsConfig {
    pub global_gemini_model: String,
    pub embedding_model: String,
    pub local_model_token_limit: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThermodynamicsConfig {
    pub entropy_explosion_threshold: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiNexusConfig {
    pub system: SystemConfig,
    pub compute: ComputeConfig,
    pub models: ModelsConfig,
    pub thermodynamics: ThermodynamicsConfig,
}

fn default_storage_path() -> String { "data/blocks".to_string() }
fn default_agent_id() -> String { "agent_mvp_001".to_string() }
fn default_admin_id() -> String { "local_admin_001".to_string() }
fn default_meta_workspace() -> String { "src/ainexus-test/meta-workspace".to_string() }

lazy_static! {
    pub static ref GLOBAL_CONFIG: Arc<AiNexusConfig> = {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("ainexus"))
            .build()
            .expect("Failed to build config from ainexus.yaml");

        Arc::new(settings.try_deserialize::<AiNexusConfig>().expect("Failed to parse ainexus.yaml"))
    };
}

pub fn get_config() -> Arc<AiNexusConfig> {
    GLOBAL_CONFIG.clone()
}

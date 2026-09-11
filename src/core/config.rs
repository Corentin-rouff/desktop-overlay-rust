use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum ProviderConfig {
    Downdetector { company_id: u64 },
    Http { url: String, expected_status: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub provider: ProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub polling_seconds: u64,
    pub service_widget_position: WidgetPosition,
    pub services: Vec<ServiceConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            polling_seconds: 30,
            service_widget_position: WidgetPosition { x: 30.0, y: 30.0 },
            services: vec![
                ServiceConfig {
                    id: "cloudflare-http".into(),
                    name: "Cloudflare (test HTTP)".into(),
                    enabled: true,
                    provider: ProviderConfig::Http {
                        url: "https://www.cloudflare.com/cdn-cgi/trace".into(),
                        expected_status: 200,
                    },
                },
                ServiceConfig {
                    id: "google-http".into(),
                    name: "Google (test HTTP)".into(),
                    enabled: true,
                    provider: ProviderConfig::Http {
                        url: "https://www.google.com/generate_204".into(),
                        expected_status: 204,
                    },
                },
            ],
        }
    }
}

impl AppConfig {
    pub fn path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("overlay-config.json")))
            .unwrap_or_else(|| PathBuf::from("overlay-config.json"))
    }

    pub fn load_or_default() -> Self {
        let path = Self::path();
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(Self::path(), json)?;
        Ok(())
    }
}

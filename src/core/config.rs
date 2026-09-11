use crate::core::widget_registry::{definition, WIDGET_DEFINITIONS};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSettings {
    pub enabled: bool,
    pub position: WidgetPosition,
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
#[serde(default)]
pub struct ServiceWidgetOptions {
    pub show_details: bool,
    pub show_last_check: bool,
    pub alerts_enabled: bool,
}

impl Default for ServiceWidgetOptions {
    fn default() -> Self {
        Self {
            show_details: true,
            show_last_check: true,
            alerts_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockOptions {
    pub show_seconds: bool,
    pub show_date: bool,
}

impl Default for ClockOptions {
    fn default() -> Self {
        Self {
            show_seconds: true,
            show_date: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemMonitorOptions {
    pub refresh_ms: u64,
    pub show_cpu: bool,
    pub show_ram: bool,
    pub show_uptime: bool,
    pub show_gpu: bool,
    pub show_disks: bool,
    pub show_disk_io: bool,
}

impl Default for SystemMonitorOptions {
    fn default() -> Self {
        Self {
            refresh_ms: 1000,
            show_cpu: true,
            show_ram: true,
            show_uptime: true,
            show_gpu: true,
            show_disks: true,
            show_disk_io: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub polling_seconds: u64,
    pub widgets: BTreeMap<String, WidgetSettings>,
    pub service_widget: ServiceWidgetOptions,
    pub clock: ClockOptions,
    pub system_monitor: SystemMonitorOptions,
    pub services: Vec<ServiceConfig>,

    // Compatibilité avec les anciennes versions du projet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_widget_position: Option<WidgetPosition>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            polling_seconds: 30,
            widgets: default_widget_map(),
            service_widget: ServiceWidgetOptions::default(),
            clock: ClockOptions::default(),
            system_monitor: SystemMonitorOptions::default(),
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
            service_widget_position: None,
        }
    }
}

fn default_widget_map() -> BTreeMap<String, WidgetSettings> {
    WIDGET_DEFINITIONS
        .iter()
        .map(|definition| {
            (
                definition.id.to_owned(),
                WidgetSettings {
                    enabled: definition.default_enabled,
                    position: WidgetPosition {
                        x: definition.default_x,
                        y: definition.default_y,
                    },
                },
            )
        })
        .collect()
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
        let mut config = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Self>(&raw).ok())
            .unwrap_or_default();

        config.ensure_widget_defaults();

        if let Some(legacy_position) = config.service_widget_position.take() {
            if let Some(service_widget) = config.widgets.get_mut("service_monitor") {
                service_widget.position = legacy_position;
            }
        }

        config.system_monitor.refresh_ms = config.system_monitor.refresh_ms.clamp(250, 10_000);
        config.polling_seconds = config.polling_seconds.clamp(5, 3600);
        config
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(Self::path(), json)?;
        Ok(())
    }

    pub fn ensure_widget_defaults(&mut self) {
        for definition in WIDGET_DEFINITIONS {
            self.widgets
                .entry(definition.id.to_owned())
                .or_insert_with(|| WidgetSettings {
                    enabled: definition.default_enabled,
                    position: WidgetPosition {
                        x: definition.default_x,
                        y: definition.default_y,
                    },
                });
        }
    }

    pub fn widget_enabled(&self, id: &str) -> bool {
        self.widgets.get(id).map(|w| w.enabled).unwrap_or(false)
    }

    pub fn widget_settings(&self, id: &str) -> Option<&WidgetSettings> {
        self.widgets.get(id)
    }

    pub fn widget_settings_mut(&mut self, id: &str) -> &mut WidgetSettings {
        self.widgets.entry(id.to_owned()).or_insert_with(|| {
            let definition = definition(id).expect("widget id must exist in registry");
            WidgetSettings {
                enabled: definition.default_enabled,
                position: WidgetPosition {
                    x: definition.default_x,
                    y: definition.default_y,
                },
            }
        })
    }

    pub fn reset_widget(&mut self, id: &str) {
        if let Some(definition) = definition(id) {
            let settings = self.widget_settings_mut(id);
            settings.enabled = definition.default_enabled;
            settings.position = WidgetPosition {
                x: definition.default_x,
                y: definition.default_y,
            };
        }
    }
}

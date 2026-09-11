use crate::{
    core::config::{ProviderConfig, ServiceConfig},
    providers,
};
use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Operational,
    Degraded,
    Outage,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ServiceSnapshot {
    pub id: String,
    pub state: HealthState,
    pub detail: String,
    pub checked_at_unix: u64,
}

pub enum MonitorCommand {
    ReplaceServices(Vec<ServiceConfig>),
    SetDowndetectorToken(String),
    SetPollingSeconds(u64),
    RefreshNow,
    Shutdown,
}

pub struct MonitorHandle {
    pub command_tx: Sender<MonitorCommand>,
    pub event_rx: Receiver<ServiceSnapshot>,
}

impl MonitorHandle {
    pub fn start(services: Vec<ServiceConfig>, polling_seconds: u64, token: String) -> Self {
        let (command_tx, command_rx) = mpsc::channel::<MonitorCommand>();
        let (event_tx, event_rx) = mpsc::channel::<ServiceSnapshot>();

        thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(8))
                .user_agent("desktop-overlay-rust/0.1")
                .build()
                .expect("Unable to build HTTP client");

            let mut services = services;
            let mut polling_seconds = polling_seconds.clamp(5, 3600);
            let mut token = token;
            let mut force_refresh = true;

            loop {
                if force_refresh {
                    for service in services.iter().filter(|s| s.enabled) {
                        let snapshot = check_service(&client, service, &token);
                        let _ = event_tx.send(snapshot);
                    }
                    force_refresh = false;
                }

                match command_rx.recv_timeout(Duration::from_secs(polling_seconds)) {
                    Ok(MonitorCommand::ReplaceServices(value)) => {
                        services = value;
                        force_refresh = true;
                    }
                    Ok(MonitorCommand::SetDowndetectorToken(value)) => {
                        token = value;
                        force_refresh = true;
                    }
                    Ok(MonitorCommand::SetPollingSeconds(value)) => {
                        polling_seconds = value.clamp(5, 3600);
                    }
                    Ok(MonitorCommand::RefreshNow) => force_refresh = true,
                    Ok(MonitorCommand::Shutdown) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => force_refresh = true,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Self {
            command_tx,
            event_rx,
        }
    }
}

pub fn empty_snapshot_map() -> HashMap<String, ServiceSnapshot> {
    HashMap::new()
}

fn check_service(
    client: &reqwest::blocking::Client,
    service: &ServiceConfig,
    token: &str,
) -> ServiceSnapshot {
    let result = match &service.provider {
        ProviderConfig::Downdetector { company_id } => {
            providers::downdetector::check(client, *company_id, token)
        }
        ProviderConfig::Http {
            url,
            expected_status,
        } => providers::http::check(client, url, *expected_status),
    };

    let (state, detail) = match result {
        Ok(v) => v,
        Err(err) => (HealthState::Unknown, err.to_string()),
    };

    ServiceSnapshot {
        id: service.id.clone(),
        state,
        detail,
        checked_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }
}

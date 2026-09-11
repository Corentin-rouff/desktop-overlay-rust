use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

#[derive(Debug, Clone, Default)]
pub struct GpuSnapshot {
    pub usage_percent: Option<f32>,
    pub name: Option<String>,
}

pub struct GpuMonitor {
    rx: Receiver<GpuSnapshot>,
    latest: GpuSnapshot,
}

impl GpuMonitor {
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();

        #[cfg(target_os = "windows")]
        thread::spawn(move || run_windows_worker(tx));

        #[cfg(not(target_os = "windows"))]
        thread::spawn(move || {
            let _ = tx.send(GpuSnapshot::default());
        });

        Self {
            rx,
            latest: GpuSnapshot::default(),
        }
    }

    pub fn refresh_latest(&mut self) -> &GpuSnapshot {
        while let Ok(snapshot) = self.rx.try_recv() {
            self.latest = snapshot;
        }
        &self.latest
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GpuEngineRow {
    name: String,
    utilization_percentage: u64,
}

#[cfg(target_os = "windows")]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VideoControllerRow {
    name: String,
}

#[cfg(target_os = "windows")]
fn run_windows_worker(tx: std::sync::mpsc::Sender<GpuSnapshot>) {
    use wmi::WMIConnection;

    let connection = match WMIConnection::new() {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("[gpu] WMI indisponible: {err}");
            let _ = tx.send(GpuSnapshot::default());
            return;
        }
    };

    let gpu_name = connection
        .raw_query::<VideoControllerRow>("SELECT Name FROM Win32_VideoController")
        .ok()
        .and_then(|rows| {
            let names = rows
                .into_iter()
                .map(|row| row.name)
                .filter(|name| {
                    let lower = name.to_ascii_lowercase();
                    !lower.contains("microsoft basic")
                        && !lower.contains("remote display")
                        && !lower.contains("indirect display")
                })
                .collect::<Vec<_>>();

            if names.is_empty() {
                None
            } else {
                Some(names.join(" + "))
            }
        });

    loop {
        let usage_percent = connection
            .raw_query::<GpuEngineRow>(
                "SELECT Name, UtilizationPercentage \
                 FROM Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine",
            )
            .ok()
            .and_then(|rows| aggregate_gpu_usage(&rows));

        if tx
            .send(GpuSnapshot {
                usage_percent,
                name: gpu_name.clone(),
            })
            .is_err()
        {
            break;
        }

        thread::sleep(Duration::from_millis(1000));
    }
}

#[cfg(target_os = "windows")]
fn aggregate_gpu_usage(rows: &[GpuEngineRow]) -> Option<f32> {
    use std::collections::HashMap;

    let mut by_engine: HashMap<String, f64> = HashMap::new();

    for row in rows {
        let lower = row.name.to_ascii_lowercase();

        // Chiffre principal proche du graphe "GPU" du Gestionnaire des tâches :
        // on s'intéresse aux moteurs graphiques / compute, pas à VideoDecode/Copy.
        if !(lower.contains("engtype_3d")
            || lower.contains("engtype_graphics")
            || lower.contains("engtype_compute")
            || lower.contains("engtype_cuda"))
        {
            continue;
        }

        // Les compteurs WDDM sont répétés par PID. Tout ce qui suit "luid_"
        // identifie l'adaptateur + le moteur : on additionne les PID qui
        // partagent ce moteur puis on garde le moteur le plus occupé.
        let Some(luid_index) = lower.find("luid_") else {
            continue;
        };

        let key = lower[luid_index..].to_owned();
        *by_engine.entry(key).or_default() += row.utilization_percentage as f64;
    }

    by_engine
        .values()
        .copied()
        .reduce(f64::max)
        .map(|value| value.clamp(0.0, 100.0) as f32)
}

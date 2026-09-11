use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

#[derive(Debug, Clone, Default)]
pub struct DiskPerfSnapshot {
    /// Pourcentage de temps pendant lequel le disque (ou l'ensemble des disques)
    /// traite activement des requêtes. `None` si Windows ne fournit pas le compteur.
    pub activity_percent: Option<f32>,
    pub read_bytes_per_sec: Option<f64>,
    pub written_bytes_per_sec: Option<f64>,
}

pub struct DiskPerfMonitor {
    rx: Receiver<DiskPerfSnapshot>,
    latest: DiskPerfSnapshot,
}

impl DiskPerfMonitor {
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();

        #[cfg(target_os = "windows")]
        thread::spawn(move || run_windows_worker(tx));

        #[cfg(not(target_os = "windows"))]
        thread::spawn(move || {
            let _ = tx.send(DiskPerfSnapshot::default());
        });

        Self {
            rx,
            latest: DiskPerfSnapshot::default(),
        }
    }

    pub fn refresh_latest(&mut self) -> &DiskPerfSnapshot {
        while let Ok(snapshot) = self.rx.try_recv() {
            self.latest = snapshot;
        }
        &self.latest
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PhysicalDiskRow {
    name: String,
    percent_disk_time: u64,
    disk_read_bytes_persec: u64,
    disk_write_bytes_persec: u64,
}

#[cfg(target_os = "windows")]
fn run_windows_worker(tx: std::sync::mpsc::Sender<DiskPerfSnapshot>) {
    use wmi::WMIConnection;

    let connection = match WMIConnection::new() {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("[disk] WMI indisponible: {err}");
            let _ = tx.send(DiskPerfSnapshot::default());
            return;
        }
    };

    loop {
        let snapshot = connection
            .raw_query::<PhysicalDiskRow>(
                "SELECT Name, PercentDiskTime, DiskReadBytesPersec, DiskWriteBytesPersec \
                 FROM Win32_PerfFormattedData_PerfDisk_PhysicalDisk",
            )
            .ok()
            .and_then(|rows| aggregate_disk_performance(&rows))
            .unwrap_or_default();

        if tx.send(snapshot).is_err() {
            break;
        }

        thread::sleep(Duration::from_millis(1000));
    }
}

#[cfg(target_os = "windows")]
fn aggregate_disk_performance(rows: &[PhysicalDiskRow]) -> Option<DiskPerfSnapshot> {
    if rows.is_empty() {
        return None;
    }

    // Windows expose normalement une instance `_Total`. Elle est idéale pour
    // un widget synthétique. Si elle n'existe pas, on prend le disque physique
    // le plus occupé et on additionne les débits disponibles.
    if let Some(total) = rows.iter().find(|row| row.name.eq_ignore_ascii_case("_Total")) {
        return Some(DiskPerfSnapshot {
            activity_percent: Some((total.percent_disk_time as f32).clamp(0.0, 100.0)),
            read_bytes_per_sec: Some(total.disk_read_bytes_persec as f64),
            written_bytes_per_sec: Some(total.disk_write_bytes_persec as f64),
        });
    }

    let activity = rows
        .iter()
        .map(|row| row.percent_disk_time as f32)
        .reduce(f32::max)
        .map(|value| value.clamp(0.0, 100.0));

    let read = rows
        .iter()
        .map(|row| row.disk_read_bytes_persec as f64)
        .sum::<f64>();
    let written = rows
        .iter()
        .map(|row| row.disk_write_bytes_persec as f64)
        .sum::<f64>();

    Some(DiskPerfSnapshot {
        activity_percent: activity,
        read_bytes_per_sec: Some(read),
        written_bytes_per_sec: Some(written),
    })
}

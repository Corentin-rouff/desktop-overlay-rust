use crate::core::{
    disk_stats::{DiskPerfMonitor, DiskPerfSnapshot},
    gpu_stats::{GpuMonitor, GpuSnapshot},
};
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Default)]
pub struct DiskSnapshot {
    pub label: String,
    pub total_space: u64,
    pub used_space: u64,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub used_memory: u64,
    pub total_memory: u64,
    pub uptime_seconds: u64,
    pub hostname: String,
    pub gpu: GpuSnapshot,
    pub disk_perf: DiskPerfSnapshot,
    pub disks: Vec<DiskSnapshot>,
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            used_memory: 0,
            total_memory: 0,
            uptime_seconds: 0,
            hostname: System::host_name().unwrap_or_else(|| "Windows".to_owned()),
            gpu: GpuSnapshot::default(),
            disk_perf: DiskPerfSnapshot::default(),
            disks: Vec::new(),
        }
    }
}

pub struct SystemStats {
    system: System,
    disks: Disks,
    gpu: GpuMonitor,
    disk_perf: DiskPerfMonitor,
    snapshot: SystemSnapshot,
    last_refresh: Instant,
}

impl SystemStats {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_memory();
        system.refresh_cpu_usage();

        let disks = Disks::new_with_refreshed_list();

        let mut this = Self {
            system,
            disks,
            gpu: GpuMonitor::start(),
            disk_perf: DiskPerfMonitor::start(),
            snapshot: SystemSnapshot::default(),
            last_refresh: Instant::now() - Duration::from_secs(1),
        };

        this.refresh();
        this
    }

    pub fn refresh_if_due(&mut self, every: Duration) {
        // Les workers WMI tournent indépendamment et ne bloquent jamais egui.
        self.snapshot.gpu = self.gpu.refresh_latest().clone();
        self.snapshot.disk_perf = self.disk_perf.refresh_latest().clone();

        if self.last_refresh.elapsed() >= every {
            self.refresh();
        }
    }

    pub fn snapshot(&self) -> &SystemSnapshot {
        &self.snapshot
    }

    fn refresh(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.disks.refresh(true);

        self.snapshot.cpu_usage = self.system.global_cpu_usage().clamp(0.0, 100.0);
        self.snapshot.used_memory = self.system.used_memory();
        self.snapshot.total_memory = self.system.total_memory();
        self.snapshot.uptime_seconds = System::uptime();
        self.snapshot.gpu = self.gpu.refresh_latest().clone();
        self.snapshot.disk_perf = self.disk_perf.refresh_latest().clone();

        self.snapshot.disks = self
            .disks
            .list()
            .iter()
            .filter(|disk| disk.total_space() > 0)
            .filter(|disk| !disk.is_removable())
            .map(|disk| {
                let total_space = disk.total_space();
                let used_space = total_space.saturating_sub(disk.available_space());
                let label = disk
                    .mount_point()
                    .to_string_lossy()
                    .trim_end_matches('\\')
                    .to_owned();

                DiskSnapshot {
                    label: if label.is_empty() {
                        disk.name().to_string_lossy().to_string()
                    } else {
                        label
                    },
                    total_space,
                    used_space,
                }
            })
            .collect();

        self.last_refresh = Instant::now();
    }
}

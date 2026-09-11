use crate::core::system_stats::SystemSnapshot;
use eframe::egui;

pub fn show(
    ctx: &egui::Context,
    position: egui::Pos2,
    edit_mode: bool,
    snapshot: &SystemSnapshot,
    show_cpu: bool,
    show_ram: bool,
    show_uptime: bool,
    show_gpu: bool,
    show_disks: bool,
    show_disk_io: bool,
) -> egui::Pos2 {
    let area = egui::Area::new(egui::Id::new("system_monitor_widget"))
        .current_pos(position)
        .movable(edit_mode)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(15, 20, 28, 238))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_premultiplied(87, 111, 145, 120),
                ))
                .corner_radius(16)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.set_min_width(310.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("SYSTÈME")
                                .strong()
                                .size(14.0)
                                .color(egui::Color32::from_rgb(216, 228, 244)),
                        );
                        if edit_mode {
                            ui.label(
                                egui::RichText::new("• déplacer")
                                    .small()
                                    .color(egui::Color32::from_gray(135)),
                            );
                        }
                    });

                    ui.label(
                        egui::RichText::new(&snapshot.hostname)
                            .small()
                            .color(egui::Color32::from_rgb(132, 151, 177)),
                    );
                    ui.add_space(10.0);

                    if show_cpu {
                        metric_bar(
                            ui,
                            "CPU",
                            snapshot.cpu_usage / 100.0,
                            format!("{:.0}%", snapshot.cpu_usage),
                        );
                    }

                    if show_gpu {
                        match snapshot.gpu.usage_percent {
                            Some(usage) => {
                                let label = snapshot
                                    .gpu
                                    .name
                                    .as_deref()
                                    .unwrap_or("GPU")
                                    .split(" + ")
                                    .next()
                                    .unwrap_or("GPU");

                                metric_bar(
                                    ui,
                                    "GPU",
                                    usage / 100.0,
                                    format!("{usage:.0}%"),
                                );
                                ui.label(
                                    egui::RichText::new(label)
                                        .small()
                                        .color(egui::Color32::from_gray(145)),
                                );
                                ui.add_space(5.0);
                            }
                            None => {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("GPU").strong());
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new("indisponible")
                                                    .small()
                                                    .color(egui::Color32::from_gray(125)),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(7.0);
                            }
                        }
                    }

                    if show_ram {
                        let memory_ratio = if snapshot.total_memory == 0 {
                            0.0
                        } else {
                            snapshot.used_memory as f32 / snapshot.total_memory as f32
                        };

                        metric_bar(
                            ui,
                            "RAM",
                            memory_ratio,
                            format!(
                                "{} / {}",
                                format_bytes(snapshot.used_memory),
                                format_bytes(snapshot.total_memory)
                            ),
                        );
                    }

                    if show_disks {
                        ui.add_space(2.0);
                        ui.separator();
                        ui.add_space(7.0);
                        ui.label(
                            egui::RichText::new("DISQUES")
                                .strong()
                                .small()
                                .color(egui::Color32::from_rgb(165, 185, 210)),
                        );
                        ui.add_space(6.0);

                        match snapshot.disk_perf.activity_percent {
                            Some(activity) => {
                                metric_bar(
                                    ui,
                                    "Activité",
                                    activity / 100.0,
                                    format!("{activity:.0}%"),
                                );
                            }
                            None => {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Activité").strong());
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new("indisponible")
                                                    .small()
                                                    .color(egui::Color32::from_gray(125)),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(7.0);
                            }
                        }

                        if show_disk_io {
                            if let (Some(read), Some(written)) = (
                                snapshot.disk_perf.read_bytes_per_sec,
                                snapshot.disk_perf.written_bytes_per_sec,
                            ) {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Lecture  {}    •    Écriture  {}",
                                        format_rate(read),
                                        format_rate(written)
                                    ))
                                    .small()
                                    .color(egui::Color32::from_gray(140)),
                                );
                                ui.add_space(8.0);
                            }
                        }

                        for disk in snapshot.disks.iter().take(4) {
                            let ratio = if disk.total_space == 0 {
                                0.0
                            } else {
                                disk.used_space as f32 / disk.total_space as f32
                            };

                            metric_bar(
                                ui,
                                &disk.label,
                                ratio,
                                format!(
                                    "{} / {}",
                                    format_bytes(disk.used_space),
                                    format_bytes(disk.total_space)
                                ),
                            );
                        }
                    }

                    if show_uptime {
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(7.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "Uptime  {}",
                                format_uptime(snapshot.uptime_seconds)
                            ))
                            .small()
                            .color(egui::Color32::from_gray(150)),
                        );
                    }
                });
        });

    area.response.rect.min
}

fn metric_bar(ui: &mut egui::Ui, label: &str, ratio: f32, text: String) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(&text)
                    .small()
                    .color(egui::Color32::from_rgb(183, 199, 219)),
            );
        });
    });

    ui.add(
        egui::ProgressBar::new(ratio.clamp(0.0, 1.0))
            .desired_width(282.0)
            .show_percentage(),
    );
    ui.add_space(7.0);
}

fn format_bytes(bytes: u64) -> String {
    const TIB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;

    let value = bytes as f64;
    if value >= TIB {
        format!("{:.1} Tio", value / TIB)
    } else if value >= GIB {
        format!("{:.1} Gio", value / GIB)
    } else {
        format!("{:.0} Mio", value / MIB)
    }
}

fn format_rate(bytes_per_sec: f64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    const KIB: f64 = 1024.0;

    if bytes_per_sec >= GIB {
        format!("{:.1} Gio/s", bytes_per_sec / GIB)
    } else if bytes_per_sec >= MIB {
        format!("{:.1} Mio/s", bytes_per_sec / MIB)
    } else if bytes_per_sec >= KIB {
        format!("{:.0} Kio/s", bytes_per_sec / KIB)
    } else {
        format!("{:.0} o/s", bytes_per_sec)
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days} j {hours} h {minutes} min")
    } else if hours > 0 {
        format!("{hours} h {minutes} min")
    } else {
        format!("{minutes} min")
    }
}

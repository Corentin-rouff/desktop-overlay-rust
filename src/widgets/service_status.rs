use crate::core::{
    config::ServiceConfig,
    monitor::{HealthState, ServiceSnapshot},
};
use eframe::egui;
use std::collections::HashMap;

pub fn show(
    ctx: &egui::Context,
    position: egui::Pos2,
    edit_mode: bool,
    services: &[ServiceConfig],
    snapshots: &HashMap<String, ServiceSnapshot>,
) -> egui::Pos2 {
    let area = egui::Area::new(egui::Id::new("service_status_widget"))
        .current_pos(position)
        .movable(edit_mode)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(218))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(64)))
                .corner_radius(14)
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.set_min_width(330.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SERVICE MONITOR").strong().size(16.0));
                        if edit_mode {
                            ui.label(
                                egui::RichText::new("  • déplacer")
                                    .small()
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        }
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let mut shown = 0usize;
                    for service in services.iter().filter(|s| s.enabled) {
                        shown += 1;
                        let snapshot = snapshots.get(&service.id);
                        let state = snapshot.map(|s| s.state).unwrap_or(HealthState::Unknown);
                        let detail = snapshot
                            .map(|s| s.detail.as_str())
                            .unwrap_or("En attente de la première vérification…");
                        service_row(ui, &service.name, state, detail);
                    }

                    if shown == 0 {
                        ui.label("Aucun service activé.");
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(if edit_mode {
                            "F8 : verrouiller l’overlay"
                        } else {
                            "F8 : mode édition"
                        })
                        .small()
                        .color(egui::Color32::GRAY),
                    );
                });
        });

    area.response.rect.min
}

fn service_row(ui: &mut egui::Ui, name: &str, state: HealthState, detail: &str) {
    let (dot, label, color) = match state {
        HealthState::Operational => ("●", "Opérationnel", egui::Color32::from_rgb(65, 210, 125)),
        HealthState::Degraded => ("●", "Perturbations", egui::Color32::from_rgb(245, 185, 60)),
        HealthState::Outage => ("●", "Panne", egui::Color32::from_rgb(240, 75, 80)),
        HealthState::Unknown => ("●", "Inconnu", egui::Color32::from_gray(150)),
    };

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(dot).size(19.0).color(color));
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(name).strong());
                ui.label(egui::RichText::new(label).small().color(color));
            });
            ui.label(
                egui::RichText::new(detail)
                    .small()
                    .color(egui::Color32::from_gray(175)),
            );
        });
    });
    ui.add_space(8.0);
}

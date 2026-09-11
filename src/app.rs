use crate::{
    core::{
        config::{AppConfig, ProviderConfig, ServiceConfig},
        hotkey,
        monitor::{
            HealthState, MonitorCommand, MonitorHandle, ServiceSnapshot, empty_snapshot_map,
        },
        windows_overlay,
    },
    widgets,
};
use eframe::egui;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

struct Alert {
    title: String,
    message: String,
    state: HealthState,
    expires_at: Instant,
}

pub struct OverlayApp {
    config: AppConfig,
    monitor: MonitorHandle,
    snapshots: HashMap<String, ServiceSnapshot>,
    edit_mode: bool,
    show_settings: bool,
    downdetector_token: String,
    last_f8_down: bool,
    config_dirty: bool,
    last_config_save: Instant,
    alerts: Vec<Alert>,

    new_name: String,
    new_company_id: String,
    new_http_url: String,
    add_mode: usize,
}

impl OverlayApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load_or_default();
        let token = std::env::var("DOWNDETECTOR_TOKEN").unwrap_or_default();
        let monitor = MonitorHandle::start(
            config.services.clone(),
            config.polling_seconds,
            token.clone(),
        );

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::from_rgb(20, 20, 24);
        cc.egui_ctx.set_visuals(visuals);

        // L'overlay démarre verrouillé : transparent et cliquable au travers.
        windows_overlay::configure_creation_context(cc, true);

        Self {
            config,
            monitor,
            snapshots: empty_snapshot_map(),
            edit_mode: false,
            show_settings: false,
            downdetector_token: token,
            last_f8_down: false,
            config_dirty: false,
            last_config_save: Instant::now(),
            alerts: Vec::new(),
            new_name: String::new(),
            new_company_id: String::new(),
            new_http_url: String::new(),
            add_mode: 0,
        }
    }

    fn set_edit_mode(&mut self, ctx: &egui::Context, frame: &eframe::Frame, enabled: bool) {
        self.edit_mode = enabled;
        let click_through = !enabled;

        // Ne jamais appeler ViewportCommand::MousePassthrough ici.
        // La fenêtre a été créée en click-through dans main.rs, afin que winit pose
        // WS_EX_LAYERED avant la création de la surface WGPU. F8 ne bascule ensuite
        // que WS_EX_TRANSPARENT, ce qui évite le fond blanc et la réapparition du cadre.
        windows_overlay::set_click_through(frame, click_through);

        if enabled {
            // Le style graphique de la fenêtre ne change pas : elle reste transparente,
            // maximisée et sans bordure. On demande uniquement le focus.
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        } else {
            // En quittant le mode édition, ferme aussi le panneau de paramètres.
            self.show_settings = false;
        }
    }

    fn poll_events(&mut self) {
        while let Ok(snapshot) = self.monitor.event_rx.try_recv() {
            let previous = self.snapshots.get(&snapshot.id).map(|s| s.state);

            if previous != Some(snapshot.state)
                && matches!(snapshot.state, HealthState::Degraded | HealthState::Outage)
            {
                let service_name = self
                    .config
                    .services
                    .iter()
                    .find(|s| s.id == snapshot.id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| snapshot.id.clone());

                self.alerts.push(Alert {
                    title: match snapshot.state {
                        HealthState::Outage => format!("Panne détectée : {service_name}"),
                        _ => format!("Perturbation : {service_name}"),
                    },
                    message: snapshot.detail.clone(),
                    state: snapshot.state,
                    expires_at: Instant::now() + Duration::from_secs(12),
                });
            }

            self.snapshots.insert(snapshot.id.clone(), snapshot);
        }

        self.alerts.retain(|a| a.expires_at > Instant::now());
    }

    fn send_services_to_monitor(&self) {
        let _ = self
            .monitor
            .command_tx
            .send(MonitorCommand::ReplaceServices(
                self.config.services.clone(),
            ));
    }

    fn mark_config_dirty(&mut self) {
        self.config_dirty = true;
    }

    fn save_config_if_needed(&mut self) {
        if self.config_dirty && self.last_config_save.elapsed() >= Duration::from_secs(1) {
            if let Err(err) = self.config.save() {
                eprintln!("Impossible de sauvegarder la configuration: {err:#}");
            } else {
                self.config_dirty = false;
                self.last_config_save = Instant::now();
            }
        }
    }

    fn draw_toolbar(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if !self.edit_mode {
            return;
        }

        egui::Area::new(egui::Id::new("overlay_toolbar"))
            .fixed_pos(egui::pos2(15.0, 15.0))
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(18, 18, 22))
                    .corner_radius(10)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Overlay • édition").strong());
                            if ui.button("⚙ Paramètres").clicked() {
                                self.show_settings = !self.show_settings;
                            }
                            if ui.button("↻ Actualiser").clicked() {
                                let _ = self.monitor.command_tx.send(MonitorCommand::RefreshNow);
                            }
                            if ui.button("🔒 Verrouiller (F8)").clicked() {
                                self.set_edit_mode(ctx, frame, false);
                            }
                            if ui.button("✕ Quitter").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
            });
    }

    fn draw_settings(&mut self, ctx: &egui::Context) {
        if !self.edit_mode || !self.show_settings {
            return;
        }

        let mut open = self.show_settings;
        egui::Window::new("Paramètres de l'overlay")
            .open(&mut open)
            .default_pos(egui::pos2(20.0, 80.0))
            .default_width(470.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Surveillance des services");
                ui.label("Le token Downdetector n'est conservé qu'en mémoire. Tu peux aussi utiliser la variable d'environnement DOWNDETECTOR_TOKEN.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Token :");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.downdetector_token)
                            .password(true)
                            .desired_width(280.0),
                    );
                    if response.changed() {
                        let _ = self.monitor.command_tx.send(
                            MonitorCommand::SetDowndetectorToken(self.downdetector_token.clone()),
                        );
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Intervalle :");
                    let mut seconds = self.config.polling_seconds as u32;
                    if ui
                        .add(egui::Slider::new(&mut seconds, 5..=300).suffix(" s"))
                        .changed()
                    {
                        self.config.polling_seconds = seconds as u64;
                        let _ = self.monitor.command_tx.send(MonitorCommand::SetPollingSeconds(
                            self.config.polling_seconds,
                        ));
                        self.mark_config_dirty();
                    }
                });

                ui.separator();
                ui.label(egui::RichText::new("Services configurés").strong());

                let mut changed = false;
                let mut remove_index = None;
                for (index, service) in self.config.services.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        changed |= ui.checkbox(&mut service.enabled, "").changed();
                        ui.label(&service.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Supprimer").clicked() {
                                remove_index = Some(index);
                            }
                        });
                    });
                }

                if let Some(index) = remove_index {
                    self.config.services.remove(index);
                    changed = true;
                }

                if changed {
                    self.send_services_to_monitor();
                    self.mark_config_dirty();
                }

                ui.separator();
                ui.label(egui::RichText::new("Ajouter un service").strong());
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.add_mode, 0, "Downdetector");
                    ui.selectable_value(&mut self.add_mode, 1, "HTTP");
                });
                ui.text_edit_singleline(&mut self.new_name)
                    .on_hover_text("Nom affiché dans le widget");

                if self.add_mode == 0 {
                    ui.horizontal(|ui| {
                        ui.label("Company ID :");
                        ui.text_edit_singleline(&mut self.new_company_id);
                    });
                    ui.label(egui::RichText::new("Le company_id est l'identifiant du service dans l'API Downdetector Enterprise.").small().color(egui::Color32::GRAY));
                } else {
                    ui.horizontal(|ui| {
                        ui.label("URL :");
                        ui.text_edit_singleline(&mut self.new_http_url);
                    });
                }

                if ui.button("＋ Ajouter").clicked() {
                    self.add_service();
                }

                ui.add_space(8.0);
                ui.separator();
                ui.label(format!("Configuration : {}", AppConfig::path().display()));
                ui.label("F8 bascule entre édition et overlay verrouillé/click-through.");
            });

        self.show_settings = open;
    }

    fn add_service(&mut self) {
        let name = self.new_name.trim();
        if name.is_empty() {
            return;
        }

        let slug = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let id = format!("{}-{}", slug, self.config.services.len() + 1);

        let provider = if self.add_mode == 0 {
            let Ok(company_id) = self.new_company_id.trim().parse::<u64>() else {
                return;
            };
            ProviderConfig::Downdetector { company_id }
        } else {
            let url = self.new_http_url.trim();
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return;
            }
            ProviderConfig::Http {
                url: url.to_owned(),
                expected_status: 200,
            }
        };

        self.config.services.push(ServiceConfig {
            id,
            name: name.to_owned(),
            enabled: true,
            provider,
        });

        self.new_name.clear();
        self.new_company_id.clear();
        self.new_http_url.clear();
        self.send_services_to_monitor();
        self.mark_config_dirty();
    }

    fn draw_alerts(&self, ctx: &egui::Context) {
        if self.alerts.is_empty() {
            return;
        }

        egui::Area::new(egui::Id::new("service_alerts"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 22.0))
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    for alert in self.alerts.iter().rev().take(3) {
                        let accent = match alert.state {
                            HealthState::Outage => egui::Color32::from_rgb(240, 75, 80),
                            _ => egui::Color32::from_rgb(245, 185, 60),
                        };
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(18, 18, 22))
                            .stroke(egui::Stroke::new(2.0, accent))
                            .corner_radius(12)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(420.0);
                                ui.label(egui::RichText::new(&alert.title).strong().color(accent));
                                ui.label(&alert.message);
                            });
                        ui.add_space(7.0);
                    }
                });
            });
    }
}

impl eframe::App for OverlayApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.poll_events();

        let f8 = hotkey::f8_pressed_edge(&mut self.last_f8_down)
            || ctx.input(|i| i.key_pressed(egui::Key::F8));
        if f8 {
            self.set_edit_mode(&ctx, frame, !self.edit_mode);
        }

        self.draw_toolbar(&ctx, frame);
        self.draw_settings(&ctx);

        let old_pos = egui::pos2(
            self.config.service_widget_position.x,
            self.config.service_widget_position.y,
        );
        let new_pos = widgets::service_status::show(
            &ctx,
            old_pos,
            self.edit_mode,
            &self.config.services,
            &self.snapshots,
        );

        if self.edit_mode && new_pos.distance(old_pos) > 0.5 {
            self.config.service_widget_position.x = new_pos.x;
            self.config.service_widget_position.y = new_pos.y;
            self.mark_config_dirty();
        }

        self.draw_alerts(&ctx);
        self.save_config_if_needed();

        ctx.request_repaint_after(Duration::from_millis(100));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        windows_overlay::clear_color()
    }

    fn on_exit(&mut self) {
        let _ = self.config.save();
        let _ = self.monitor.command_tx.send(MonitorCommand::Shutdown);
    }
}

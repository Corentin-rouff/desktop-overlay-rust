use crate::{
    core::{
        config::{AppConfig, ProviderConfig, ServiceConfig},
        monitor::{empty_snapshot_map, HealthState, MonitorCommand, MonitorHandle, ServiceSnapshot},
        system_stats::SystemStats,
        tray::{TrayAction, TrayController},
        widget_registry::{CLOCK_ID, SERVICE_MONITOR_ID, SYSTEM_MONITOR_ID, WIDGET_DEFINITIONS},
        windows_overlay,
    },
    widgets,
};
use eframe::egui;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsPage {
    General,
    Widgets,
    System,
    Services,
}

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
    system_stats: SystemStats,
    tray: Option<TrayController>,
    edit_mode: bool,
    show_settings: bool,
    focus_settings: bool,
    reopen_settings_after_edit: bool,
    settings_page: SettingsPage,
    downdetector_token: String,
    config_dirty: bool,
    last_config_save: Instant,
    alerts: Vec<Alert>,

    new_name: String,
    new_company_id: String,
    new_http_url: String,
    add_mode: usize,
}


fn sidebar_button(
    ui: &mut egui::Ui,
    current: &mut SettingsPage,
    page: SettingsPage,
    icon: &str,
    label: &str,
) {
    let selected = *current == page;
    let caption = if icon.is_empty() {
        label.to_owned()
    } else {
        format!("{icon}   {label}")
    };
    let text = if selected {
        egui::RichText::new(caption)
            .strong()
            .color(egui::Color32::from_rgb(225, 237, 250))
    } else {
        egui::RichText::new(caption)
            .color(egui::Color32::from_rgb(154, 171, 194))
    };

    let response = ui.add_sized(
        [ui.available_width(), 38.0],
        egui::Button::new(text)
            .fill(if selected {
                egui::Color32::from_rgb(28, 51, 77)
            } else {
                egui::Color32::TRANSPARENT
            })
            .stroke(egui::Stroke::NONE)
            .corner_radius(8),
    );

    if response.clicked() {
        *current = page;
    }
}

fn settings_card(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(18, 24, 34))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(42, 55, 73),
        ))
        .corner_radius(12)
        .inner_margin(12.0)
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_width(width);
            ui.label(
                egui::RichText::new(title)
                    .strong()
                    .color(egui::Color32::from_rgb(211, 224, 241)),
            );
            ui.add_space(7.0);
            add_contents(ui);
        });
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

        let tray = match TrayController::new(cc.egui_ctx.clone()) {
            Ok(tray) => Some(tray),
            Err(err) => {
                eprintln!("[tray] Impossible de créer l’icône de zone de notification: {err:#}");
                None
            }
        };
        let tray_failed = tray.is_none();

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::from_rgb(20, 20, 24);
        cc.egui_ctx.set_visuals(visuals);

        // Normalement l'overlay démarre verrouillé. Si le tray échoue, on ouvre
        // les paramètres en mode interactif pour ne jamais enfermer l'utilisateur.
        // La fenêtre racine reste click-through au démarrage, même si le tray échoue.
        // La fenêtre Paramètres est un viewport natif séparé et reste interactive.
        windows_overlay::configure_creation_context(cc, true);

        Self {
            config,
            monitor,
            snapshots: empty_snapshot_map(),
            system_stats: SystemStats::new(),
            tray,
            edit_mode: false,
            show_settings: tray_failed,
            focus_settings: tray_failed,
            reopen_settings_after_edit: false,
            settings_page: SettingsPage::General,
            downdetector_token: token,
            config_dirty: false,
            last_config_save: Instant::now(),
            alerts: Vec::new(),
            new_name: String::new(),
            new_company_id: String::new(),
            new_http_url: String::new(),
            add_mode: 0,
        }
    }

    fn set_edit_mode(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
        enabled: bool,
    ) {
        self.edit_mode = enabled;
        windows_overlay::set_click_through(frame, !enabled);

        if let Some(tray) = &self.tray {
            tray.set_edit_mode(enabled);
        }

        if enabled {
            // L'overlay racine devient interactif : la fenêtre Paramètres doit être
            // masquée pour ne pas se retrouver derrière la grande fenêtre always-on-top.
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
        } else if self.reopen_settings_after_edit {
            // Si l'édition a été lancée depuis Paramètres, on revient automatiquement
            // dans Paramètres une fois la disposition reverrouillée.
            self.reopen_settings_after_edit = false;
            self.show_settings = true;
            self.focus_settings = true;
        }
    }

    fn start_layout_edit_from_settings(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
    ) {
        self.show_settings = false;
        self.focus_settings = false;
        self.reopen_settings_after_edit = true;
        self.set_edit_mode(ctx, frame, true);
    }

    fn poll_tray(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let actions = self
            .tray
            .as_mut()
            .map(|tray| tray.poll_actions())
            .unwrap_or_default();

        for action in actions {
            match action {
                TrayAction::Settings => {
                    // Les paramètres ne doivent jamais être ouverts pendant que
                    // l'overlay plein écran capture la souris.
                    if self.edit_mode {
                        self.reopen_settings_after_edit = false;
                        self.set_edit_mode(ctx, frame, false);
                    }
                    self.settings_page = SettingsPage::General;
                    self.show_settings = true;
                    self.focus_settings = true;
                }
                TrayAction::ToggleEdit => {
                    if self.edit_mode {
                        self.set_edit_mode(ctx, frame, false);
                    } else {
                        self.show_settings = false;
                        self.focus_settings = false;
                        self.reopen_settings_after_edit = false;
                        self.set_edit_mode(ctx, frame, true);
                    }
                }
                TrayAction::Refresh => {
                    let _ = self.monitor.command_tx.send(MonitorCommand::RefreshNow);
                }
                TrayAction::Quit => {
                    ctx.send_viewport_cmd_to(
                        egui::ViewportId::ROOT,
                        egui::ViewportCommand::Close,
                    );
                }
            }
        }
    }

    fn poll_events(&mut self) {
        while let Ok(snapshot) = self.monitor.event_rx.try_recv() {
            let previous = self.snapshots.get(&snapshot.id).map(|s| s.state);

            if self.config.service_widget.alerts_enabled
                && previous != Some(snapshot.state)
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
            .send(MonitorCommand::ReplaceServices(self.config.services.clone()));
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
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 16.0))
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(13, 18, 26, 246))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_premultiplied(75, 112, 158, 150),
                    ))
                    .corner_radius(14)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Mode édition")
                                    .strong()
                                    .color(egui::Color32::from_rgb(225, 234, 246)),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new("Déplace les widgets puis termine l’édition")
                                    .small()
                                    .color(egui::Color32::from_gray(155)),
                            );
                            ui.separator();

                            if ui.button("Terminer et verrouiller").clicked() {
                                self.set_edit_mode(ctx, frame, false);
                            }

                            if ui.button("Paramètres").clicked() {
                                self.reopen_settings_after_edit = false;
                                self.set_edit_mode(ctx, frame, false);
                                self.settings_page = SettingsPage::General;
                                self.show_settings = true;
                                self.focus_settings = true;
                            }
                        });
                    });
            });
    }

    fn draw_settings(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if !self.show_settings {
            return;
        }

        let viewport_id = egui::ViewportId::from_hash_of("desktop_overlay_settings");
        let mut close_requested = false;
        let mut start_edit_requested = false;

        ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title("Desktop Overlay — Paramètres")
                .with_inner_size([1040.0, 720.0])
                .with_min_inner_size([880.0, 620.0])
                .with_resizable(true)
                .with_decorations(true)
                .with_transparent(false)
                .with_taskbar(true)
                .with_maximize_button(false),
            |ui, _class| {
                if ui.input(|input| input.viewport().close_requested()) {
                    close_requested = true;
                    return;
                }

                ui.set_min_size(ui.available_size());

                // Une vraie sidebar à largeur fixe empêche les textes de se compresser
                // verticalement lorsque la fenêtre est redimensionnée.
                egui::Panel::left("desktop_overlay_settings_sidebar")
                    .exact_size(224.0)
                    .resizable(false)
                    .show_separator_line(true)
                    .frame(
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(14, 19, 28))
                            .inner_margin(16.0),
                    )
                    .show(ui, |ui| {
                        self.draw_settings_sidebar(ui);
                    });

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(10, 14, 21))
                            .inner_margin(egui::Margin::symmetric(22, 18)),
                    )
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(self.settings_title())
                                .size(25.0)
                                .strong()
                                .color(egui::Color32::from_rgb(229, 237, 248)),
                        );
                        ui.add_space(3.0);
                        ui.label(
                            egui::RichText::new(self.settings_subtitle())
                                .color(egui::Color32::from_rgb(132, 151, 177)),
                        );
                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(12.0);

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                match self.settings_page {
                                    SettingsPage::General => {
                                        start_edit_requested |= self.draw_general_settings(ui);
                                    }
                                    SettingsPage::Widgets => self.draw_widget_settings(ui),
                                    SettingsPage::System => self.draw_system_settings(ui),
                                    SettingsPage::Services => self.draw_service_settings(ui),
                                }
                            });
                    });
            },
        );

        if self.focus_settings {
            ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::Focus);
            self.focus_settings = false;
        }

        if close_requested {
            self.show_settings = false;
        }

        if start_edit_requested {
            self.start_layout_edit_from_settings(ctx, frame);
        }
    }

    fn draw_settings_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.set_min_height(ui.available_height());

        ui.label(
            egui::RichText::new("DESKTOP OVERLAY")
                .strong()
                .size(15.0)
                .color(egui::Color32::from_rgb(92, 176, 255)),
        );
        ui.label(
            egui::RichText::new("Control Center")
                .small()
                .color(egui::Color32::from_gray(125)),
        );
        ui.add_space(24.0);

        sidebar_button(ui, &mut self.settings_page, SettingsPage::General, "", "Général");
        ui.add_space(4.0);
        sidebar_button(ui, &mut self.settings_page, SettingsPage::Widgets, "", "Widgets");
        ui.add_space(4.0);
        sidebar_button(ui, &mut self.settings_page, SettingsPage::System, "", "Système");
        ui.add_space(4.0);
        sidebar_button(ui, &mut self.settings_page, SettingsPage::Services, "", "Services");

        let footer_height = 70.0;
        let remaining = ui.available_height();
        if remaining > footer_height {
            ui.add_space(remaining - footer_height);
        }

        ui.separator();
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("v0.3")
                .small()
                .color(egui::Color32::from_gray(90)),
        );
        ui.label(
            egui::RichText::new("Contrôle via l’icône de zone de notification")
                .small()
                .color(egui::Color32::from_gray(110)),
        );
    }

    fn settings_title(&self) -> &'static str {
        match self.settings_page {
            SettingsPage::General => "Général",
            SettingsPage::Widgets => "Widgets",
            SettingsPage::System => "Monitoring système",
            SettingsPage::Services => "Services",
        }
    }

    fn settings_subtitle(&self) -> &'static str {
        match self.settings_page {
            SettingsPage::General => "Comportement de l’overlay et informations de l’application",
            SettingsPage::Widgets => "Active, désactive et réinitialise les modules du bureau",
            SettingsPage::System => "CPU, GPU, mémoire, stockage et fréquence de rafraîchissement",
            SettingsPage::Services => "Surveillance HTTP et Downdetector",
        }
    }

    fn draw_general_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut start_edit_requested = false;

        settings_card(ui, "État de l’overlay", |ui| {
            let (dot, text, detail) = if self.edit_mode {
                (
                    egui::Color32::from_rgb(245, 185, 60),
                    "Mode édition actif",
                    "Les widgets sont interactifs et peuvent être déplacés.",
                )
            } else {
                (
                    egui::Color32::from_rgb(80, 210, 160),
                    "Overlay verrouillé",
                    "Les clics traversent l’overlay vers les applications en dessous.",
                )
            };

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("●").color(dot));
                ui.label(egui::RichText::new(text).strong());
            });
            ui.add_space(3.0);
            ui.label(
                egui::RichText::new(detail)
                    .small()
                    .color(egui::Color32::from_gray(145)),
            );

            ui.add_space(12.0);

            if !self.edit_mode
                && ui
                    .button("Modifier la disposition des widgets")
                    .clicked()
            {
                start_edit_requested = true;
            }
        });

        ui.add_space(10.0);

        settings_card(ui, "Zone de notification Windows", |ui| {
            ui.label("Le contrôle principal se fait depuis l’icône Desktop Overlay près de l’horloge Windows.");
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new(
                    "Clic droit : Paramètres, Modifier la disposition, Actualiser les services ou Quitter.",
                )
                .small()
                .color(egui::Color32::from_gray(145)),
            );
            ui.add_space(3.0);
            ui.label(
                egui::RichText::new(
                    "Quand tu modifies la disposition, cette fenêtre se ferme temporairement. Une barre de contrôle reste visible en haut de l’écran pour terminer l’édition, puis les paramètres se rouvrent automatiquement.",
                )
                .small()
                .color(egui::Color32::from_gray(145)),
            );
        });

        ui.add_space(10.0);

        settings_card(ui, "Configuration", |ui| {
            ui.label(
                egui::RichText::new("Fichier de configuration")
                    .small()
                    .color(egui::Color32::from_gray(135)),
            );
            ui.add_space(3.0);
            ui.label(
                egui::RichText::new(AppConfig::path().display().to_string())
                    .monospace()
                    .small()
                    .color(egui::Color32::from_rgb(158, 181, 211)),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Les modifications sont sauvegardées automatiquement.")
                    .small()
                    .color(egui::Color32::from_gray(135)),
            );
        });

        start_edit_requested
    }

    fn draw_widget_settings(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            if ui.button("✓  Tout activer").clicked() {
                for definition in WIDGET_DEFINITIONS {
                    self.config.widget_settings_mut(definition.id).enabled = true;
                }
                changed = true;
            }
            if ui.button("Tout désactiver").clicked() {
                for definition in WIDGET_DEFINITIONS {
                    self.config.widget_settings_mut(definition.id).enabled = false;
                }
                changed = true;
            }
        });
        ui.add_space(10.0);

        for definition in WIDGET_DEFINITIONS {
            let settings = self.config.widget_settings_mut(definition.id);
            settings_card(ui, definition.name, |ui| {
                ui.horizontal(|ui| {
                    changed |= ui.checkbox(&mut settings.enabled, "Activé").changed();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Réinitialiser").clicked() {
                            settings.enabled = definition.default_enabled;
                            settings.position.x = definition.default_x;
                            settings.position.y = definition.default_y;
                            changed = true;
                        }
                    });
                });
                ui.label(
                    egui::RichText::new(definition.description)
                        .small()
                        .color(egui::Color32::from_gray(145)),
                );
            });
            ui.add_space(8.0);
        }

        settings_card(ui, "Horloge — affichage", |ui| {
            let enabled = self.config.widget_enabled(CLOCK_ID);
            ui.add_enabled_ui(enabled, |ui| {
                changed |= ui.checkbox(&mut self.config.clock.show_seconds, "Afficher les secondes").changed();
                changed |= ui.checkbox(&mut self.config.clock.show_date, "Afficher la date").changed();
            });
        });

        if changed {
            self.mark_config_dirty();
        }
    }

    fn draw_system_settings(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        let enabled = self.config.widget_enabled(SYSTEM_MONITOR_ID);

        settings_card(ui, "Contenu du widget", |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                changed |= ui
                    .checkbox(&mut self.config.system_monitor.show_cpu, "Afficher l’utilisation CPU")
                    .changed();
                changed |= ui
                    .checkbox(&mut self.config.system_monitor.show_gpu, "Afficher l’utilisation GPU")
                    .changed();
                changed |= ui
                    .checkbox(&mut self.config.system_monitor.show_ram, "Afficher l’utilisation RAM")
                    .changed();
                changed |= ui
                    .checkbox(&mut self.config.system_monitor.show_disks, "Afficher l’utilisation des disques")
                    .changed();
                if self.config.system_monitor.show_disks {
                    changed |= ui
                        .checkbox(&mut self.config.system_monitor.show_disk_io, "Afficher les débits lecture / écriture")
                        .changed();
                }
                changed |= ui
                    .checkbox(&mut self.config.system_monitor.show_uptime, "Afficher l’uptime")
                    .changed();
            });
        });

        ui.add_space(10.0);

        settings_card(ui, "Rafraîchissement", |ui| {
            let mut refresh_ms = self.config.system_monitor.refresh_ms as u32;
            if ui
                .add(
                    egui::Slider::new(&mut refresh_ms, 250..=5000)
                        .step_by(250.0)
                        .suffix(" ms")
                        .text("Intervalle"),
                )
                .changed()
            {
                self.config.system_monitor.refresh_ms = refresh_ms as u64;
                changed = true;
            }

            ui.label(
                egui::RichText::new(
                    "Le GPU est collecté dans un worker WMI séparé afin de ne pas bloquer l’interface.",
                )
                .small()
                .color(egui::Color32::from_gray(135)),
            );
        });

        ui.add_space(10.0);

        let snapshot = self.system_stats.snapshot();
        settings_card(ui, "Détection actuelle", |ui| {
            ui.label(format!("CPU : {:.0} %", snapshot.cpu_usage));
            ui.label(match snapshot.gpu.usage_percent {
                Some(value) => format!(
                    "GPU : {:.0} %{}",
                    value,
                    snapshot
                        .gpu
                        .name
                        .as_deref()
                        .map(|name| format!(" — {name}"))
                        .unwrap_or_default()
                ),
                None => "GPU : donnée indisponible".to_owned(),
            });
            ui.label(format!("Disques détectés : {}", snapshot.disks.len()));
        });

        if changed {
            self.mark_config_dirty();
        }
    }

    fn draw_service_settings(&mut self, ui: &mut egui::Ui) {
        let mut widget_options_changed = false;
        settings_card(ui, "Affichage du widget Service Monitor", |ui| {
            let enabled = self.config.widget_enabled(SERVICE_MONITOR_ID);
            ui.add_enabled_ui(enabled, |ui| {
                widget_options_changed |= ui
                    .checkbox(&mut self.config.service_widget.show_details, "Afficher le détail du statut")
                    .changed();
                widget_options_changed |= ui
                    .checkbox(&mut self.config.service_widget.show_last_check, "Afficher la dernière vérification")
                    .changed();
            });
            widget_options_changed |= ui
                .checkbox(
                    &mut self.config.service_widget.alerts_enabled,
                    "Afficher les alertes de panne même si le widget est masqué",
                )
                .changed();
        });
        if widget_options_changed {
            self.mark_config_dirty();
        }

        ui.add_space(10.0);

        settings_card(ui, "Downdetector", |ui| {
            ui.label(
                egui::RichText::new(
                    "Le token reste uniquement en mémoire. La variable DOWNDETECTOR_TOKEN est également prise en charge.",
                )
                .small()
                .color(egui::Color32::from_gray(145)),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Token API");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.downdetector_token)
                        .password(true)
                        .hint_text("Bearer token Downdetector Enterprise")
                        .desired_width(420.0),
                );
                if response.changed() {
                    let _ = self.monitor.command_tx.send(
                        MonitorCommand::SetDowndetectorToken(self.downdetector_token.clone()),
                    );
                }
            });
        });

        ui.add_space(10.0);

        settings_card(ui, "Fréquence de surveillance", |ui| {
            ui.horizontal(|ui| {
                let mut seconds = self.config.polling_seconds as u32;
                if ui
                    .add(
                        egui::Slider::new(&mut seconds, 5..=300)
                            .suffix(" s")
                            .text("Intervalle"),
                    )
                    .changed()
                {
                    self.config.polling_seconds = seconds as u64;
                    let _ = self
                        .monitor
                        .command_tx
                        .send(MonitorCommand::SetPollingSeconds(self.config.polling_seconds));
                    self.mark_config_dirty();
                }

                if ui.button("↻  Vérifier maintenant").clicked() {
                    let _ = self.monitor.command_tx.send(MonitorCommand::RefreshNow);
                }
            });
        });

        ui.add_space(10.0);

        let mut services_changed = false;
        let mut remove_index = None;

        settings_card(ui, "Services surveillés", |ui| {
            if self.config.services.is_empty() {
                ui.label(
                    egui::RichText::new("Aucun service configuré.")
                        .color(egui::Color32::from_gray(140)),
                );
                return;
            }

            for (index, service) in self.config.services.iter_mut().enumerate() {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(14, 19, 27))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(38, 49, 65),
                    ))
                    .corner_radius(9)
                    .inner_margin(9.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            services_changed |= ui.checkbox(&mut service.enabled, "").changed();

                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&service.name).strong());
                                let provider = match &service.provider {
                                    ProviderConfig::Downdetector { company_id } => {
                                        format!("Downdetector • company_id {company_id}")
                                    }
                                    ProviderConfig::Http {
                                        url,
                                        expected_status,
                                    } => format!("HTTP {expected_status} • {url}"),
                                };
                                ui.label(
                                    egui::RichText::new(provider)
                                        .small()
                                        .color(egui::Color32::from_gray(135)),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("Supprimer").clicked() {
                                        remove_index = Some(index);
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
            }
        });

        if let Some(index) = remove_index {
            self.config.services.remove(index);
            services_changed = true;
        }

        if services_changed {
            self.send_services_to_monitor();
            self.mark_config_dirty();
        }

        ui.add_space(10.0);

        settings_card(ui, "Ajouter un service", |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.add_mode, 0, "Downdetector");
                ui.selectable_value(&mut self.add_mode, 1, "HTTP / Healthcheck");
            });
            ui.add_space(8.0);

            egui::Grid::new("add_service_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Nom");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_name)
                            .hint_text("Ex. Discord")
                            .desired_width(420.0),
                    );
                    ui.end_row();

                    if self.add_mode == 0 {
                        ui.label("Company ID");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_company_id)
                                .hint_text("Identifiant Downdetector")
                                .desired_width(420.0),
                        );
                        ui.end_row();
                    } else {
                        ui.label("URL");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_http_url)
                                .hint_text("https://example.com/health")
                                .desired_width(420.0),
                        );
                        ui.end_row();
                    }
                });

            ui.add_space(10.0);
            if ui.button("＋  Ajouter le service").clicked() {
                self.add_service();
            }
        });
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

    fn draw_widgets(&mut self, ctx: &egui::Context) {
        self.draw_service_widget(ctx);
        self.draw_clock_widget(ctx);
        self.draw_system_widget(ctx);
    }

    fn draw_service_widget(&mut self, ctx: &egui::Context) {
        if !self.config.widget_enabled(SERVICE_MONITOR_ID) {
            return;
        }

        let Some(settings) = self.config.widget_settings(SERVICE_MONITOR_ID).cloned() else {
            return;
        };
        let old_pos = egui::pos2(settings.position.x, settings.position.y);
        let new_pos = widgets::service_status::show(
            ctx,
            old_pos,
            self.edit_mode,
            &self.config.services,
            &self.snapshots,
            self.config.service_widget.show_details,
            self.config.service_widget.show_last_check,
        );
        self.store_widget_position(SERVICE_MONITOR_ID, old_pos, new_pos);
    }

    fn draw_clock_widget(&mut self, ctx: &egui::Context) {
        if !self.config.widget_enabled(CLOCK_ID) {
            return;
        }

        let Some(settings) = self.config.widget_settings(CLOCK_ID).cloned() else {
            return;
        };
        let old_pos = egui::pos2(settings.position.x, settings.position.y);
        let new_pos = widgets::clock::show(
            ctx,
            old_pos,
            self.edit_mode,
            self.config.clock.show_seconds,
            self.config.clock.show_date,
        );
        self.store_widget_position(CLOCK_ID, old_pos, new_pos);
    }

    fn draw_system_widget(&mut self, ctx: &egui::Context) {
        if !self.config.widget_enabled(SYSTEM_MONITOR_ID) {
            return;
        }

        let snapshot = self.system_stats.snapshot().clone();

        let Some(settings) = self.config.widget_settings(SYSTEM_MONITOR_ID).cloned() else {
            return;
        };
        let old_pos = egui::pos2(settings.position.x, settings.position.y);
        let new_pos = widgets::system_monitor::show(
            ctx,
            old_pos,
            self.edit_mode,
            &snapshot,
            self.config.system_monitor.show_cpu,
            self.config.system_monitor.show_ram,
            self.config.system_monitor.show_uptime,
            self.config.system_monitor.show_gpu,
            self.config.system_monitor.show_disks,
            self.config.system_monitor.show_disk_io,
        );
        self.store_widget_position(SYSTEM_MONITOR_ID, old_pos, new_pos);
    }

    fn store_widget_position(&mut self, id: &str, old_pos: egui::Pos2, new_pos: egui::Pos2) {
        if self.edit_mode && new_pos.distance(old_pos) > 0.5 {
            let settings = self.config.widget_settings_mut(id);
            settings.position.x = new_pos.x;
            settings.position.y = new_pos.y;
            self.mark_config_dirty();
        }
    }

    fn draw_alerts(&self, ctx: &egui::Context) {
        if !self.config.service_widget.alerts_enabled || self.alerts.is_empty() {
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
        self.poll_tray(&ctx, frame);

        if self.config.widget_enabled(SYSTEM_MONITOR_ID)
            || (self.show_settings && self.settings_page == SettingsPage::System)
        {
            let refresh = Duration::from_millis(
                self.config.system_monitor.refresh_ms.clamp(250, 10_000),
            );
            self.system_stats.refresh_if_due(refresh);
        }

        self.draw_toolbar(&ctx, frame);
        self.draw_settings(&ctx, frame);
        self.draw_widgets(&ctx);
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

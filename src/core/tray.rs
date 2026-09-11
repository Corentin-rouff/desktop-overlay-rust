use eframe::egui;

#[cfg(target_os = "windows")]
use std::sync::mpsc::{self, Receiver};

#[cfg(target_os = "windows")]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Settings,
    ToggleEdit,
    Refresh,
    Quit,
}

#[cfg(target_os = "windows")]
pub struct TrayController {
    _tray: TrayIcon,
    // On conserve explicitement les MenuItem pendant toute la durée de vie du tray.
    _settings_item: MenuItem,
    edit_item: MenuItem,
    _refresh_item: MenuItem,
    _quit_item: MenuItem,
    action_rx: Receiver<TrayAction>,
}

#[cfg(target_os = "windows")]
impl TrayController {
    pub fn new(egui_ctx: egui::Context) -> anyhow::Result<Self> {
        let settings_item =
            MenuItem::with_id("overlay.settings", "Paramètres…", true, None);
        let edit_item =
            MenuItem::with_id("overlay.edit", "Modifier la disposition", true, None);
        let refresh_item = MenuItem::with_id(
            "overlay.refresh",
            "Actualiser les services",
            true,
            None,
        );
        let quit_item = MenuItem::with_id("overlay.quit", "Quitter", true, None);

        let sep1 = PredefinedMenuItem::separator();
        let sep2 = PredefinedMenuItem::separator();

        let menu = Menu::with_items(&[
            &settings_item,
            &edit_item,
            &sep1,
            &refresh_item,
            &sep2,
            &quit_item,
        ])?;

        let tray = TrayIconBuilder::new()
            .with_tooltip("Desktop Overlay")
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true)
            .with_icon(build_icon()?)
            .build()?;

        // IMPORTANT : tray-icon recommande un event handler avec winit/eframe.
        // Le callback est appelé immédiatement lorsqu'un élément du menu est activé,
        // puis réveille egui afin que l'action soit traitée sans attendre un input
        // souris/clavier sur la fenêtre principale (qui est normalement click-through).
        let (action_tx, action_rx) = mpsc::channel::<TrayAction>();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let id: &str = event.id.as_ref();
            let action = match id {
                "overlay.settings" => Some(TrayAction::Settings),
                "overlay.edit" => Some(TrayAction::ToggleEdit),
                "overlay.refresh" => Some(TrayAction::Refresh),
                "overlay.quit" => Some(TrayAction::Quit),
                _ => None,
            };

            if let Some(action) = action {
                #[cfg(debug_assertions)]
                eprintln!("[tray] action reçue : {action:?}");

                let _ = action_tx.send(action);
                egui_ctx.request_repaint();
            }
        }));

        Ok(Self {
            _tray: tray,
            _settings_item: settings_item,
            edit_item,
            _refresh_item: refresh_item,
            _quit_item: quit_item,
            action_rx,
        })
    }

    pub fn poll_actions(&mut self) -> Vec<TrayAction> {
        let mut actions = Vec::new();
        while let Ok(action) = self.action_rx.try_recv() {
            actions.push(action);
        }
        actions
    }

    pub fn set_edit_mode(&self, enabled: bool) {
        self.edit_item.set_text(if enabled {
            "Verrouiller les widgets"
        } else {
            "Modifier la disposition"
        });
    }
}

#[cfg(target_os = "windows")]
impl Drop for TrayController {
    fn drop(&mut self) {
        // Le handler est global dans tray-icon : on le retire proprement à la fermeture.
        MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
    }
}

#[cfg(target_os = "windows")]
fn build_icon() -> anyhow::Result<Icon> {
    const W: u32 = 32;
    const H: u32 = 32;
    let mut rgba = vec![0_u8; (W * H * 4) as usize];

    for y in 0..H {
        for x in 0..W {
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let distance = (dx * dx + dy * dy).sqrt();
            let idx = ((y * W + x) * 4) as usize;

            if distance <= 14.0 {
                rgba[idx] = 44;
                rgba[idx + 1] = 151;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;

                if (7..=24).contains(&x) && (9..=20).contains(&y) {
                    rgba[idx] = 16;
                    rgba[idx + 1] = 24;
                    rgba[idx + 2] = 38;
                }
                if (10..=21).contains(&x) && (12..=17).contains(&y) {
                    rgba[idx] = 78;
                    rgba[idx + 1] = 208;
                    rgba[idx + 2] = 170;
                }
                if (13..=18).contains(&x) && (21..=23).contains(&y) {
                    rgba[idx] = 230;
                    rgba[idx + 1] = 238;
                    rgba[idx + 2] = 248;
                }
            }
        }
    }

    Ok(Icon::from_rgba(rgba, W, H)?)
}

#[cfg(not(target_os = "windows"))]
pub struct TrayController;

#[cfg(not(target_os = "windows"))]
impl TrayController {
    pub fn new(_egui_ctx: egui::Context) -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub fn poll_actions(&mut self) -> Vec<TrayAction> {
        Vec::new()
    }

    pub fn set_edit_mode(&self, _enabled: bool) {}
}

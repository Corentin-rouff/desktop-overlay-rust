mod app;
mod core;
mod providers;
mod widgets;

use app::OverlayApp;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Desktop Overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_has_shadow(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_maximized(true)
            .with_resizable(false)
            // IMPORTANT : le click-through est demandé DÈS LA CRÉATION.
            // Sous Windows, winit pose alors WS_EX_LAYERED avant que WGPU
            // n'installe sa surface. On ne changera ensuite que WS_EX_TRANSPARENT.
            .with_mouse_passthrough(true),

        renderer: eframe::Renderer::Wgpu,
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        "Desktop Overlay",
        options,
        Box::new(|cc| Ok(Box::new(OverlayApp::new(cc)))),
    )
}

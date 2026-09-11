#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// La fenêtre est créée avec `with_mouse_passthrough(true)` dans main.rs.
///
/// Sous Windows, winit crée alors la fenêtre avec les styles nécessaires au
/// click-through (notamment WS_EX_LAYERED + WS_EX_TRANSPARENT) AVANT la création
/// de la surface graphique. C'est important : ajouter WS_EX_LAYERED après coup
/// peut casser la composition alpha de WGPU et produire un grand fond blanc.
///
/// Une fois la fenêtre créée, nous ne modifions donc PLUS JAMAIS WS_EX_LAYERED.
/// F8 ne change que WS_EX_TRANSPARENT.
#[cfg(target_os = "windows")]
pub fn configure_creation_context(cc: &eframe::CreationContext<'_>, click_through: bool) {
    apply_click_through_only(cc, click_through);
}

#[cfg(not(target_os = "windows"))]
pub fn configure_creation_context(_cc: &eframe::CreationContext<'_>, _click_through: bool) {}

/// Active/désactive le click-through sans reconstruire le style de fenêtre.
///
/// IMPORTANT :
/// - ne touche PAS à WS_EX_LAYERED ;
/// - ne touche PAS à GWL_STYLE ;
/// - ne touche PAS aux décorations ;
/// - n'utilise PAS ViewportCommand::MousePassthrough à chaud.
#[cfg(target_os = "windows")]
pub fn set_click_through(frame: &eframe::Frame, click_through: bool) {
    apply_click_through_only(frame, click_through);
}

#[cfg(not(target_os = "windows"))]
pub fn set_click_through(_frame: &eframe::Frame, _click_through: bool) {}

/// Fond réellement transparent pour eframe/WGPU.
/// À combiner avec `ViewportBuilder::with_transparent(true)`.
pub fn clear_color() -> [f32; 4] {
    eframe::egui::Rgba::TRANSPARENT.to_array()
}

#[cfg(target_os = "windows")]
fn apply_click_through_only<T: HasWindowHandle>(window: &T, click_through: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW, WS_EX_TRANSPARENT,
    };

    let Ok(window_handle) = window.window_handle() else {
        eprintln!("[overlay] Impossible d'obtenir le handle natif de la fenêtre.");
        return;
    };

    let RawWindowHandle::Win32(win32) = window_handle.as_raw() else {
        return;
    };

    let hwnd = win32.hwnd.get() as *mut core::ffi::c_void;

    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let transparent_flag = WS_EX_TRANSPARENT as isize;

        let next = if click_through {
            current | transparent_flag
        } else {
            current & !transparent_flag
        };

        if next != current {
            // On change UNIQUEMENT le bit WS_EX_TRANSPARENT.
            // Pas de SWP_FRAMECHANGED : il n'y a aucun cadre à recalculer.
            let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next);
        }
    }
}

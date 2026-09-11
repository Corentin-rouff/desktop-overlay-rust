#[cfg(target_os = "windows")]
pub fn f8_pressed_edge(last_down: &mut bool) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F8};

    let down = unsafe { GetAsyncKeyState(VK_F8 as i32) < 0 };
    let pressed = down && !*last_down;
    *last_down = down;
    pressed
}

#[cfg(not(target_os = "windows"))]
pub fn f8_pressed_edge(_last_down: &mut bool) -> bool {
    false
}

//! Windows 11 chrome: Mica backdrop, DWM shadow into the client area and the
//! rounded corners (was `apply_chrome` in main.rs).

#[cfg(windows)]
pub fn apply(app: &crate::AppWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use slint::ComponentHandle;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };

    let handle = app.window().window_handle();
    let _ = window_vibrancy::apply_mica(&handle, Some(true));

    let Ok(handle) = handle.window_handle() else { return };
    let RawWindowHandle::Win32(win) = handle.as_raw() else { return };
    let hwnd = HWND(win.hwnd.get() as _);

    unsafe {
        #[repr(C)]
        struct Margins {
            cx_left_width: i32,
            cx_right_width: i32,
            cy_top_height: i32,
            cy_bottom_height: i32,
        }
        #[link(name = "dwmapi")]
        unsafe extern "system" {
            fn DwmExtendFrameIntoClientArea(hwnd: HWND, margins: *const Margins) -> i32;
        }
        let _ = DwmExtendFrameIntoClientArea(
            hwnd,
            &Margins {
                cx_left_width: 1,
                cx_right_width: 1,
                cy_top_height: 1,
                cy_bottom_height: 1,
            },
        );

        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &DWMWCP_ROUND as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&DWMWCP_ROUND) as u32,
        );
    }
}

#[cfg(not(windows))]
pub fn apply(_: &crate::AppWindow) {}

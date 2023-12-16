use crate::windows;
use std::{cell::Cell, mem};
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::{SetLastError, ERROR_SUCCESS, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Performance::QueryPerformanceCounter},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, RegisterClassExW,
            SetWindowLongPtrW, UnregisterClassW, CW_USEDEFAULT, GWLP_USERDATA, HMENU, HWND_MESSAGE,
            WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW,
        },
    },
};

use crate::error::ResultExt;

thread_local! {
    static NEXT_WINDOW_USER_DATA_ON_INIT: Cell<isize> = const { Cell::new(0) };
}

/// The window procedure of a class that you must define.
///
/// Parameters without types: `hwnd, msg_id, wparam, lparam`
/// Parameters with types: `hwnd: HWND, msg_id: u32, wparam: WPARAM, lparam: LPARAM`
///
/// Return `None` to cause `DefWindowProcW()` being called and its return value being used. You sometimes should also call it yourself when handling certain messages and returning `Some(...)`.
pub type WindowProcedure<'a> = dyn FnMut(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT> + 'a;

/// A window class registered with `RegisterClassExW()`, containing a window procedure closure. Necessary for creating windows.
///
/// Don't use `Get...`/`SetWindowLongPtrW(...GWLP_USERDATA...)` on a window created from an instance of this struct, because it stores internal data necessary for the struct to function.
pub struct WindowClass<'a> {
    atom: u16,
    /// Converted with `Box::into_raw()`.
    window_procedure_ptr: *mut Box<WindowProcedure<'a>>,
}

impl<'a> WindowClass<'a> {
    pub fn new(window_procedure: Box<WindowProcedure<'a>>) -> windows::core::Result<Self> {
        let mut precise_time = 0;
        unsafe { QueryPerformanceCounter(&mut precise_time)? };

        Self::with_name(&format!("unnamed_{precise_time:x}"), window_procedure)
    }

    pub fn with_name(
        name: &str,
        window_procedure: Box<WindowProcedure<'a>>,
    ) -> windows::core::Result<Self> {
        Self::with_details(
            WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as _,
                lpfnWndProc: Some(Self::base_window_procedure),
                hInstance: unsafe { GetModuleHandleW(PCWSTR::null())? }.into(),
                lpszClassName: PCWSTR(HSTRING::from(name).as_ptr()),
                ..Default::default()
            },
            window_procedure,
        )
    }

    pub fn with_details(
        mut wnd_class_ex: WNDCLASSEXW,
        window_procedure: Box<WindowProcedure<'a>>,
    ) -> windows::core::Result<Self> {
        //! The `lpfnWndProc` field will be overwritten.

        wnd_class_ex.lpfnWndProc = Some(Self::base_window_procedure);
        let atom = Result::from_nonzero_or_win32(unsafe { RegisterClassExW(&wnd_class_ex) })?;

        Ok(Self {
            atom,
            // Double indirection to get thin pointer.
            window_procedure_ptr: Box::into_raw(Box::new(window_procedure)),
        })
    }

    pub fn atom(&self) -> u16 {
        self.atom
    }

    extern "system" fn base_window_procedure(
        hwnd: HWND,
        msg_id: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // Retrieve saved window procedure.
        let mut user_data = unsafe {
            SetLastError(ERROR_SUCCESS);
            GetWindowLongPtrW(hwnd, GWLP_USERDATA)
        };

        // On first message, save window procedure for subsequent calls. This is the first time that the `HWND` is known.
        if user_data == 0 {
            //. Consume value, so failing once below makes for failing on subsequent calls (until `CreateWindowExW()` was aborted).
            user_data = NEXT_WINDOW_USER_DATA_ON_INIT.replace(0);

            let result = Result::<(), windows::core::Error>::from_win32().and_then(|_| unsafe {
                SetLastError(ERROR_SUCCESS);
                Result::from_nonzero_and_win32(SetWindowLongPtrW(hwnd, GWLP_USERDATA, user_data))
            });

            if result.is_err() {
                // Make `CreateWindowExW()` fail.
                // (First message may be `WM_GETMINMAXINFO`, then, `WM_NCCREATE` is expected, which still happens during the `CreateWindowExW()` call. `LRESULT(0)` indicates an error for `WM_NCCREATE`, while it indicates success for `WM_GETMINMAXINFO` and many other messages.)
                return LRESULT(0);
            }
        };

        // Call window procedure.
        let window_procedure = unsafe { &mut *(user_data as *mut Box<WindowProcedure>) };

        if let Some(lresult) = window_procedure(hwnd, msg_id, wparam, lparam) {
            lresult
        } else {
            // Call default message handler.
            unsafe { DefWindowProcW(hwnd, msg_id, wparam, lparam) }
        }
    }
}

impl Drop for WindowClass<'_> {
    fn drop(&mut self) {
        unsafe {
            if let Ok(h_module) = GetModuleHandleW(PCWSTR::null()) {
                let _ = UnregisterClassW(PCWSTR(self.atom as _), h_module);
            }

            drop(Box::from_raw(self.window_procedure_ptr));
        }
    }
}

pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn new_msg_only(class: &WindowClass) -> windows::core::Result<Self> {
        //! Creates a message-only window.
        //!
        //! See <https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#message-only-windows>.

        Self::with_details(
            class,
            Some(HWND_MESSAGE),
            WINDOW_STYLE(0),
            None,
            None,
            None,
            None,
        )
    }

    pub fn new_invisible(class: &WindowClass) -> windows::core::Result<Self> {
        //! Meant for windows that stay invisible. Necessary instead of a message-only window, if you want to receive broadcast messages like `WM_ENDSESSION`.

        Self::with_details(class, None, WINDOW_STYLE(0), None, None, None, None)
    }

    pub fn with_details(
        class: &WindowClass,
        parent: Option<HWND>,
        style: WINDOW_STYLE,
        ex_style: Option<WINDOW_EX_STYLE>,
        placement: Option<(POINT, SIZE)>,
        text: Option<PCWSTR>,
        menu: Option<HMENU>,
    ) -> windows::core::Result<Self> {
        //! Creates a window with `CreateWindowExW()`.
        //!
        //! `None` for `placement` uses `CW_USEDEFAULT` for all four values.

        // Pass window procedure via thread-local storage instead of `CREATESTRUCTW`, because `WM_GETMINMAXINFO` can be sent before `WM_NCCREATE`.
        NEXT_WINDOW_USER_DATA_ON_INIT.set(class.window_procedure_ptr as _);

        // Create window.
        let (pos, size) = placement.unwrap_or((
            POINT {
                x: CW_USEDEFAULT,
                y: CW_USEDEFAULT,
            },
            SIZE {
                cx: CW_USEDEFAULT,
                cy: CW_USEDEFAULT,
            },
        ));

        let hwnd = Result::from_checked_or_win32(
            unsafe {
                CreateWindowExW(
                    ex_style.unwrap_or(WINDOW_EX_STYLE(0)),
                    PCWSTR(class.atom as _),
                    text.unwrap_or(PCWSTR::null()),
                    style,
                    pos.x,
                    pos.y,
                    size.cx,
                    size.cy,
                    parent.unwrap_or_default(),
                    menu.unwrap_or_default(),
                    GetModuleHandleW(PCWSTR::null())?,
                    None,
                )
            },
            |hwnd| hwnd.0 != 0,
        )?;

        Ok(Self { hwnd })
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

macro_rules! hiword {
    ($param:ident) => {
        ($param.0 >> 16 & 0xffff) as u16
    };
}

macro_rules! loword {
    ($param:ident) => {
        ($param.0 & 0xffff) as u16
    };
}

pub fn translate_command_msg(wparam: WPARAM, lparam: LPARAM) -> CommandMsg {
    match hiword!(wparam) {
        0 => CommandMsg::MenuItem {
            id: loword!(wparam),
        },
        1 => CommandMsg::Accelerator {
            id: loword!(wparam),
        },
        wparam_hiword => CommandMsg::ControlMsg {
            msg_id: wparam_hiword,
            control_id: loword!(wparam),
            control_hwnd: HWND(lparam.0),
        },
    }
}

pub enum CommandMsg {
    MenuItem {
        id: u16,
    },
    Accelerator {
        id: u16,
    },
    ControlMsg {
        msg_id: u16,
        control_id: u16,
        control_hwnd: HWND,
    },
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use crate::windows;
    use std::{cell::RefCell, rc::Rc};
    use windows::{
        core::{HSTRING, PCWSTR},
        Win32::{
            Foundation::{HWND, LRESULT, POINT, SIZE},
            UI::WindowsAndMessaging::{
                MessageBoxW, PostQuitMessage, MB_OK, MINMAXINFO, WM_DESTROY, WM_GETMINMAXINFO,
                WM_LBUTTONUP, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
            },
        },
    };

    use super::{Window, WindowClass};
    use crate::win32_app::msg_loop;

    #[ignore]
    #[test]
    fn create_window() -> windows::core::Result<()> {
        let counter = Rc::new(RefCell::new(1));

        let class = WindowClass::new(Box::new(|hwnd, msg_id, wparam, lparam| {
            println!("window msg received: {hwnd:?}, msg 0x{msg_id:04x}, {wparam:?}, {lparam:?}");

            match msg_id {
                WM_LBUTTONUP => {
                    *counter.borrow_mut() += 1;

                    unsafe {
                        MessageBoxW(
                            HWND(0),
                            PCWSTR(HSTRING::from(format!("{counter:?}")).as_ptr()),
                            PCWSTR::null(),
                            MB_OK,
                        )
                    };

                    Some(LRESULT(0))
                }
                WM_GETMINMAXINFO => {
                    //TODO: `.cast()`-Funktion für `LPARAM` hinzufügen.
                    let min_max_info = unsafe { &mut *(lparam.0 as *mut MINMAXINFO) };
                    min_max_info.ptMaxTrackSize = POINT { x: 300, y: 300 };

                    Some(LRESULT(0))
                }
                WM_DESTROY => {
                    unsafe { PostQuitMessage(0) };
                    Some(LRESULT(0))
                }
                _ => None,
            }
        }))?;

        *counter.borrow_mut() += 1;

        let _window = Window::with_details(
            &class,
            None,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            None,
            Some((POINT { x: 100, y: 100 }, SIZE { cx: 500, cy: 500 })),
            Some(PCWSTR(HSTRING::from("Test Window").as_ptr())),
            None,
        );

        *counter.borrow_mut() += 1;

        msg_loop::run()?;

        Ok(())
    }
}

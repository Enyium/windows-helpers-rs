use crate::{
    core::{CheckNullError, CheckNumberError, ResultExt},
    windows, Null, Zeroed,
};
use std::{cell::Cell, mem};
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::{SetLastError, ERROR_SUCCESS, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Performance::QueryPerformanceCounter},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, IsWindow,
            RegisterClassExW, SetWindowLongPtrW, UnregisterClassW, CW_USEDEFAULT, GWLP_USERDATA,
            HMENU, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW,
        },
    },
};

mod translate;

pub use translate::*;

thread_local! {
    static NEXT_WINDOW_USER_DATA_ON_INIT: Cell<isize> = const { Cell::new(0) };
}

// For trait bounds in this API.
pub trait WndProc: FnMut(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT> {}

// For accepting any matching closure type where the trait bound is required.
impl<F> WndProc for F where F: FnMut(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT> {}

/// A window class registered with `RegisterClassExW()`, containing a window procedure closure. Necessary for creating windows.
///
/// - Don't drop it before any [`Window`]s created with it, because this tries to unregister the class (struct field order is relevant).
/// - Don't use `Get...`/`SetWindowLongPtrW(...GWLP_USERDATA...)` on a window created from an instance of this struct, because it stores internal data necessary for the struct to function.
pub struct WindowClass<'a> {
    atom: u16,
    /// Double-`Box`, converted with `Box::into_raw()` (to get thin pointer).
    wnd_proc_ptr: *mut Box<dyn WndProc + 'a>,
}

impl<'a> WindowClass<'a> {
    pub fn new<F>(wnd_proc: F) -> windows::core::Result<Self>
    where
        F: WndProc + 'a,
    {
        //! Creates a new class with a name from [`Self::make_name()`].
        //!
        //! Pass the window procedure of the class that you implement. Its parameters are:
        //!
        //! - Without types: `hwnd, msg_id, wparam, lparam`
        //! - With types: `hwnd: HWND, msg_id: u32, wparam: WPARAM, lparam: LPARAM`
        //!
        //! Return `None` from the procedure to cause [`DefWindowProcW()`][1] being called and its return value being used. You sometimes should also call it yourself when handling certain messages and returning `Some(...)`.
        //!
        //! Note that some functions like, e.g., [`DestroyWindow()`][2] and [`MoveWindow()`][3] synchronously cause the window procedure to be called again during their calls. This means that closing over an [`Rc<RefCell<...>>`](std::cell::RefCell) and calling `borrow_mut()` to call your actual procedure implementation (can be a method with self parameter) will cause a borrowing panic. Using `Rc<ReentrantRefCell<...>>` instead solves this. See [`crate::ReentrantRefCell`] for more information.
        //!
        //! [1]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defwindowprocw
        //! [2]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow
        //! [3]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-movewindow

        Self::with_name(&Self::make_name()?, wnd_proc)
    }

    pub fn with_name<F>(name: &str, wnd_proc: F) -> windows::core::Result<Self>
    where
        F: WndProc + 'a,
    {
        Self::with_details(
            WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as _,
                lpfnWndProc: Some(Self::base_wnd_proc),
                hInstance: unsafe { GetModuleHandleW(PCWSTR::NULL)? }.into(),
                lpszClassName: PCWSTR(HSTRING::from(name).as_ptr()),
                ..Default::default()
            },
            wnd_proc,
        )
    }

    pub fn with_details<F>(
        mut wnd_class_ex: WNDCLASSEXW,
        wnd_proc: F,
    ) -> windows::core::Result<Self>
    where
        F: WndProc + 'a,
    {
        //! The `lpfnWndProc` field will be overwritten.

        wnd_class_ex.lpfnWndProc = Some(Self::base_wnd_proc);

        Ok(Self {
            atom: unsafe { RegisterClassExW(&wnd_class_ex) }.nonzero_or_win32_err()?,
            // Double indirection to get thin pointer.
            wnd_proc_ptr: Box::into_raw(Box::new(Box::new(wnd_proc))),
        })
    }

    pub fn make_name() -> windows::core::Result<String> {
        //! Generates a time-based class name.

        let mut precise_time = 0;
        unsafe { QueryPerformanceCounter(&mut precise_time)? };

        Ok(format!("unnamed_{precise_time:x}"))
    }

    pub fn atom(&self) -> u16 {
        self.atom
    }

    extern "system" fn base_wnd_proc(
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
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, user_data).nonzero_with_win32_or_err()
            });

            if result.is_err() {
                // Make `CreateWindowExW()` fail.
                // (First message may be `WM_GETMINMAXINFO`, then, `WM_NCCREATE` is expected, which still happens during the `CreateWindowExW()` call. `LRESULT(0)` indicates an error for `WM_NCCREATE`, while it indicates success for `WM_GETMINMAXINFO` and many other messages.)
                return LRESULT(0);
            }
        };

        // Call window procedure.
        // (Outer box was dissolved into raw pointer, whose data is simply referenced here. The `Box` you see is the inner `Box`.)
        let wnd_proc = unsafe { &mut *(user_data as *mut Box<dyn WndProc>) };

        if let Some(lresult) = wnd_proc(hwnd, msg_id, wparam, lparam) {
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
            if let Ok(h_module) = GetModuleHandleW(PCWSTR::NULL) {
                let result = UnregisterClassW(PCWSTR(self.atom as _), h_module);
                debug_assert!(
                    result.is_ok(),
                    "couldn't unregister window class (did you adhere to proper drop order?): {result:?}"
                );
            }

            drop(Box::from_raw(self.wnd_proc_ptr));
        }
    }
}

/// A window created with a [`WindowClass`].
///
/// The first calls of the window procedure are made during the constructor call; then during the message loop.
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
        //! Meant for windows that stay invisible. Necessary instead of a message-only window, if you want to receive broadcast messages like `WM_ENDSESSION` or `RegisterWindowMessageW(w!("TaskbarCreated"))`.

        Self::with_details(
            class,
            None,
            WINDOW_STYLE(0),
            None,
            Some((POINT::zeroed(), SIZE::zeroed())),
            None,
            None,
        )
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
        NEXT_WINDOW_USER_DATA_ON_INIT.set(class.wnd_proc_ptr as _);

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

        let hwnd = unsafe {
            CreateWindowExW(
                ex_style.unwrap_or(WINDOW_EX_STYLE(0)),
                PCWSTR(class.atom as _),
                text.unwrap_or(PCWSTR::NULL),
                style,
                pos.x,
                pos.y,
                size.cx,
                size.cy,
                parent.unwrap_or(HWND::NULL),
                menu.unwrap_or(HMENU::NULL),
                GetModuleHandleW(PCWSTR::NULL)?,
                None,
            )
        }
        .nonnull_or_e_handle()?;

        Ok(Self { hwnd })
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn is_valid(&self) -> bool {
        //! Returns whether the associated `HWND` is still valid.
        //!
        //! It isn't valid anymore, if [`DestroyWindow()`][1] was called.
        //!
        //! [1]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow

        unsafe { IsWindow(self.hwnd) }.as_bool()
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // Calling it again when it was already called on the window is simply a no-op. This can regularly happen, when, e.g., `DefWindowProcW()` calls it on `WM_CLOSE`.
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use super::{Window, WindowClass};
    use crate::{foundation::LParamExt, win32_app::msg_loop, windows, Null};
    use std::{cell::RefCell, rc::Rc};
    use windows::{
        core::{w, HSTRING, PCWSTR},
        Win32::{
            Foundation::{HWND, LRESULT, POINT, SIZE},
            UI::WindowsAndMessaging::{
                MessageBoxW, PostQuitMessage, MB_OK, MINMAXINFO, WM_DESTROY, WM_GETMINMAXINFO,
                WM_LBUTTONUP, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
            },
        },
    };

    #[ignore]
    #[test]
    fn create_window() -> windows::core::Result<()> {
        let counter = Rc::new(RefCell::new(1));

        let class = WindowClass::new(|hwnd, msg_id, wparam, mut lparam| {
            println!("window msg received: {hwnd:?}, msg 0x{msg_id:04x}, {wparam:?}, {lparam:?}");

            match msg_id {
                WM_LBUTTONUP => {
                    *counter.borrow_mut() += 1;

                    unsafe {
                        MessageBoxW(
                            HWND::NULL,
                            PCWSTR(HSTRING::from(format!("{counter:?}")).as_ptr()),
                            w!("Message Box"),
                            MB_OK,
                        )
                    };

                    Some(LRESULT(0))
                }
                WM_GETMINMAXINFO => {
                    let min_max_info = unsafe { lparam.cast_to_mut::<MINMAXINFO>() };
                    min_max_info.ptMaxTrackSize = POINT { x: 300, y: 300 };

                    Some(LRESULT(0))
                }
                WM_DESTROY => {
                    unsafe { PostQuitMessage(0) };
                    Some(LRESULT(0))
                }
                _ => None,
            }
        })?;

        *counter.borrow_mut() += 1;

        let _window = Window::with_details(
            &class,
            None,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            None,
            Some((POINT { x: 100, y: 100 }, SIZE { cx: 500, cy: 500 })),
            Some(PCWSTR(HSTRING::from("Test Window").as_ptr())),
            None,
        )?;

        *counter.borrow_mut() += 1;

        msg_loop::run()?;

        Ok(())
    }
}

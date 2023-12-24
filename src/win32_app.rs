#![cfg(feature = "win32_app")]

//! Helpers to simplify some tedious aspects of a Win32 application.
//!
//! They allow you to combine them with the regular Windows API, and so can be undermined, which is why they haven't been implemented hiding underlying complexity too much. E.g., a `Window` could be destoyed via its `HWND` while the struct continues to exist.
//!
//! The types automatically free resources they allocated on drop, but you have to pay attention to the correct drop order, e.g., in structs. Since fields are dropped from top to bottom, specify the higher-level resources last.
//!
//! Activate the feature `windows_<version>_win32_app` (available from `windows` v0.52 onwards).

pub mod error;
pub mod msg_loop;
pub mod tray_icon;
pub mod window;

mod app;

pub use app::*;

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use super::{
        error::{try_or_quit_now, try_or_set_app_error, try_then_favor_app_error},
        tray_icon::SimplifiedTrayIconMsg,
        window::translate_timer_msg,
        AppLike, InvisibleWindowAppHelper,
    };
    use crate::{
        core::ResultExt,
        util::ReentrantRefCell,
        win32_app::{
            msg_loop,
            tray_icon::{BalloonIcon, TrayIcon},
        },
        windows, Null, ResGuard,
    };
    use anyhow::anyhow;
    use std::rc::Rc;
    use windows::Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        UI::WindowsAndMessaging::{
            DestroyWindow, KillTimer, LoadIconW, PostQuitMessage, SetTimer, HICON, IDI_APPLICATION,
            WM_APP, WM_CREATE, WM_DESTROY, WM_TIMER,
        },
    };

    #[ignore]
    #[test]
    fn tray_icon_app() -> anyhow::Result<()> {
        try_then_favor_app_error(|| -> anyhow::Result<()> {
            let (_app_helper, _app) = App::new()?;
            msg_loop::run()?;

            Ok(())
        })
        .map_err(|e| anyhow!(e))
    }

    struct App {
        tray_icon: TrayIcon,
        _tray_h_icon: ResGuard<HICON>,
    }

    impl App {
        const TRAY_ICON_MSG: u32 = WM_APP;
        const TIMER_ID: usize = 1;
    }

    impl<'a> AppLike<InvisibleWindowAppHelper<'a>> for App {
        fn new() -> windows::core::Result<(
            InvisibleWindowAppHelper<'a>,
            Rc<ReentrantRefCell<Option<Self>>>,
        )> {
            Ok(InvisibleWindowAppHelper::make_app()?)
        }

        fn startup_wnd_proc(
            hwnd: HWND,
            msg_id: u32,
            _wparam: WPARAM,
            _lparam: LPARAM,
        ) -> (Option<Self>, Option<LRESULT>) {
            if msg_id == WM_CREATE {
                let app = try_or_set_app_error(|| -> windows::core::Result<_> {
                    let mut tray_icon = TrayIcon::with_primary_id(hwnd, Some(Self::TRAY_ICON_MSG))?;

                    let tray_h_icon = ResGuard::with_acq_and_destroy_icon(|| unsafe {
                        LoadIconW(HINSTANCE::NULL, IDI_APPLICATION)
                    })?;
                    unsafe { tray_icon.set_icon(*tray_h_icon)? };

                    tray_icon.show(true)?;

                    Result::from_nonzero_or_win32(unsafe {
                        SetTimer(hwnd, Self::TIMER_ID, 1500 /*ms*/, None)
                    })?;

                    Ok(Self {
                        tray_icon,
                        _tray_h_icon: tray_h_icon,
                    })
                });

                if let Some(app) = app {
                    (Some(app), Some(LRESULT(0)))
                } else {
                    // Make window creation fail.
                    (None, Some(LRESULT(-1)))
                }
            } else {
                (None, None)
            }
        }

        fn wnd_proc(
            &mut self,
            hwnd: HWND,
            msg_id: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> Option<LRESULT> {
            match msg_id {
                WM_TIMER => {
                    let msg = unsafe { translate_timer_msg(wparam, lparam) };

                    if msg.timer_id == Self::TIMER_ID {
                        try_or_quit_now(|| -> windows::core::Result<()> {
                            unsafe { KillTimer(hwnd, Self::TIMER_ID)? };

                            self.tray_icon.show_balloon(
                                BalloonIcon::User,
                                Some("Title"),
                                "This is the notification message.",
                                false,
                                true,
                                true,
                            )?;

                            self.tray_icon.set_tooltip(Some("Click the icon to exit"))?;

                            Ok(())
                        });
                    }

                    Some(LRESULT(0))
                }
                Self::TRAY_ICON_MSG => {
                    match self
                        .tray_icon
                        .simplifying_translate_window_msg(wparam, lparam)
                    {
                        SimplifiedTrayIconMsg::Activated => {
                            try_or_quit_now(|| -> windows::core::Result<_> {
                                self.tray_icon.delete()?;
                                unsafe { DestroyWindow(hwnd) }
                            });
                            Some(LRESULT(0))
                        }
                        _ => None,
                    }
                }
                WM_DESTROY => {
                    unsafe { PostQuitMessage(0) };
                    Some(LRESULT(0))
                }
                _ => None,
            }
        }
    }
}

use crate::{
    bit_manipulation::{build_bit_flag_set, Width32BitPortion},
    core::HStringExt,
    foundation::BoolExt,
    windows, Null,
};
use map_self::MapSelf;
use std::{
    mem,
    time::{Duration, Instant},
};
use windows::{
    core::{GUID, HSTRING},
    Win32::{
        Foundation::{HWND, LPARAM, RECT, WPARAM},
        UI::{
            Input::KeyboardAndMouse::GetDoubleClickTime,
            Shell::{
                Shell_NotifyIconGetRect, Shell_NotifyIconW, NIF_GUID, NIF_ICON, NIF_INFO,
                NIF_MESSAGE, NIF_REALTIME, NIF_SHOWTIP, NIF_STATE, NIF_TIP, NIIF_ERROR, NIIF_INFO,
                NIIF_LARGE_ICON, NIIF_NONE, NIIF_NOSOUND, NIIF_RESPECT_QUIET_TIME, NIIF_USER,
                NIIF_WARNING, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETFOCUS, NIM_SETVERSION,
                NINF_KEY, NIN_SELECT, NIS_HIDDEN, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
                NOTIFYICONIDENTIFIER, NOTIFYICON_VERSION_4, NOTIFY_ICON_DATA_FLAGS,
                NOTIFY_ICON_INFOTIP_FLAGS,
            },
            WindowsAndMessaging::{HICON, WM_CONTEXTMENU},
        },
    },
};

//TODO: Constant expected to be available in `windows` v0.53. See <https://github.com/microsoft/win32metadata/issues/1765>.
const NIN_KEYSELECT: u32 = NIN_SELECT | NINF_KEY;

/// An abstraction over `Shell_NotifyIconW()`.
///
/// The icon is initially hidden and must be shown with `show()`.
///
/// To avoid fetching a low-quality icon, the app's manifest must declare it as fully DPI-aware (or jump through other hoops to get an appropriately sized icon).
pub struct TrayIcon {
    notify_icon_data: NOTIFYICONDATAW,
    last_activation_time: Instant,
}

impl TrayIcon {
    pub fn with_primary_id(hwnd: HWND, window_msg_id: Option<u32>) -> windows::core::Result<Self> {
        //! Creates a tray icon with ID 0. If you need more than one tray icon, don't use this function repeatedly.

        Self::with_id(0, hwnd, window_msg_id)
    }

    pub fn with_id(id: u16, hwnd: HWND, window_msg_id: Option<u32>) -> windows::core::Result<Self> {
        Self::with_details(Some(id), None, hwnd, window_msg_id)
    }

    pub fn with_guid(
        guid: GUID,
        hwnd: HWND,
        window_msg_id: Option<u32>,
    ) -> windows::core::Result<Self> {
        //! Creates a tray icon identified by a GUID.
        //!
        //! Microsoft recommends this over the ID approach. Things like changing the executable path may, however, make a later call to this function with an unchanged GUID fail. See <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw#troubleshooting>.

        Self::with_details(None, Some(guid), hwnd, window_msg_id)
    }

    fn with_details(
        id: Option<u16>,
        guid: Option<GUID>,
        hwnd: HWND,
        window_msg_id: Option<u32>,
    ) -> windows::core::Result<Self> {
        let notify_icon_data = NOTIFYICONDATAW {
            cbSize: mem::size_of::<NOTIFYICONDATAW>() as _,
            hWnd: hwnd,
            // `id` has to be `u16`. See docs of `uCallbackMessage` field.
            uID: id.unwrap_or_default() as _,
            uFlags: NIF_STATE
                | NIF_INFO
                | build_bit_flag_set([
                    (guid.is_some(), NIF_GUID),
                    (window_msg_id.is_some(), NIF_MESSAGE),
                ]),
            uCallbackMessage: window_msg_id.unwrap_or_default(),
            hIcon: HICON::NULL,
            szTip: [0; 128],
            dwState: NIS_HIDDEN,
            dwStateMask: NIS_HIDDEN.0,
            szInfo: [0; 256],
            Anonymous: NOTIFYICONDATAW_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            szInfoTitle: [0; 64],
            dwInfoFlags: NOTIFY_ICON_INFOTIP_FLAGS(0),
            guidItem: guid.unwrap_or(GUID::zeroed()),
            hBalloonIcon: HICON::NULL,
        };

        if guid.is_some() {
            // If the app is forcefully terminated, so that is can't call the `NIM_DELETE` command, the icon continues to linger in the tray until a mouse-move event. If this event doesn't occur and the app is restarted, `NIM_ADD` without this previous `NIM_DELETE` would fail. (The icon with the GUID seems to still count as registered and alive.)
            unsafe { Shell_NotifyIconW(NIM_DELETE, &notify_icon_data) };
        }

        let mut inst = Self {
            notify_icon_data,
            last_activation_time: Instant::now()
                .map_self_or_keep(|now| now.checked_sub(Duration::from_secs(60))),
        };

        inst.readd()?;
        inst.notify_icon_data.dwStateMask = 0;

        Ok(inst)
    }

    pub fn readd(&self) -> windows::core::Result<()> {
        //! Adds the icon again.
        //!
        //! Only to be called when receiving the window message `RegisterWindowMessageW(w!("TaskbarCreated"))`, which is also sent when `explorer.exe` restarted.

        unsafe {
            for action in [NIM_ADD, NIM_SETVERSION] {
                Shell_NotifyIconW(action, &self.notify_icon_data)
                    .ok_or_e_fail()
                    .or_else(|error| {
                        Shell_NotifyIconW(NIM_DELETE, &self.notify_icon_data);
                        Err(error)
                    })?;
            }
        }

        Ok(())
    }

    pub fn rect(&self) -> windows::core::Result<RECT> {
        //! Calls `Shell_NotifyIconGetRect()`.

        unsafe {
            Shell_NotifyIconGetRect(&NOTIFYICONIDENTIFIER {
                cbSize: mem::size_of::<NOTIFYICONIDENTIFIER>() as _,
                hWnd: self.notify_icon_data.hWnd,
                uID: self.notify_icon_data.uID,
                guidItem: if (self.notify_icon_data.uFlags & NIF_GUID).0 != 0 {
                    self.notify_icon_data.guidItem
                } else {
                    //TODO: See <https://github.com/microsoft/windows-rs/issues/2752>.
                    GUID::zeroed()
                },
            })
        }
    }

    pub unsafe fn set_icon(&mut self, h_icon: HICON) -> windows::core::Result<()> {
        //! Sets a new icon.
        //!
        //! # Safety
        //! You are responsibile that the icon is valid and to only free it after it has been replaced or this [`TrayIcon`] has been dropped.

        self.notify_icon_data.uFlags |= NIF_ICON;
        self.notify_icon_data.hIcon = h_icon;

        self.call_modify()
    }

    pub fn set_tooltip<T>(&mut self, tooltip: Option<T>) -> windows::core::Result<()>
    where
        T: Into<HSTRING>,
    {
        //! Sets the tooltip text shown when hovering over the tray icon.
        //!
        //! Long text will be truncated. See <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw>.

        const FLAGS: NOTIFY_ICON_DATA_FLAGS = NOTIFY_ICON_DATA_FLAGS(NIF_TIP.0 | NIF_SHOWTIP.0);

        if let Some(tooltip) = tooltip {
            let hstring: HSTRING = tooltip.into();

            self.notify_icon_data.uFlags |= FLAGS;
            hstring.write_truncated(&mut self.notify_icon_data.szTip);
        } else {
            self.notify_icon_data.uFlags &= !FLAGS;
        }

        self.call_modify()
    }

    pub fn show(&mut self, show: bool) -> windows::core::Result<()> {
        //TODO: Change expected in `windows` v0.53. More uses than just here. See <https://github.com/microsoft/win32metadata/issues/1767>.
        if show {
            self.notify_icon_data.dwState.0 &= !NIS_HIDDEN.0;
        } else {
            self.notify_icon_data.dwState.0 |= NIS_HIDDEN.0;
        }
        self.notify_icon_data.dwStateMask = NIS_HIDDEN.0;

        let result = self.call_modify();
        self.notify_icon_data.dwStateMask = 0;

        result
    }

    pub fn is_shown(&self) -> bool {
        (self.notify_icon_data.dwState.0 & NIS_HIDDEN.0) == 0
    }

    pub fn focus<T>(&mut self) -> windows::core::Result<()> {
        //! Performs the `NIM_SETFOCUS` command.
        //!
        //! See <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw>.

        unsafe { Shell_NotifyIconW(NIM_SETFOCUS, &self.notify_icon_data).ok_or_e_fail() }
    }

    pub unsafe fn set_balloon_icon(&mut self, h_icon: Option<HICON>) -> windows::core::Result<()> {
        //! Sets a new icon for balloon notifications, when shown with [`BalloonIcon::User`].
        //!
        //! `None`, which is the default, means the same icon as the tray icon.
        //!
        //! # Safety
        //! See [`Self::set_icon()`].

        self.notify_icon_data.hBalloonIcon = h_icon.unwrap_or(HICON::NULL);
        self.call_modify()
    }

    pub fn set_balloon_uses_large_icon(&mut self, uses_large_icon: bool) {
        //! Sets the `NIIF_LARGE_ICON` flag.
        //!
        //! May not change the size of the displayed icon.
        //!
        //! See <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw#niif_large_icon-0x00000020>.

        if uses_large_icon {
            self.notify_icon_data.dwInfoFlags.0 |= NIIF_LARGE_ICON.0;
        } else {
            self.notify_icon_data.dwInfoFlags.0 &= !NIIF_LARGE_ICON.0;
        }
    }

    pub fn show_balloon<T>(
        &mut self,
        icon: BalloonIcon,
        title: Option<T>,
        text: T,
        realtime_only: bool,
        override_quiet_time: bool,
        allow_sound: bool,
    ) -> windows::core::Result<()>
    where
        T: Into<HSTRING>,
    {
        //! Shows a so-called balloon notification that will automatically be hidden after a while. Not really a balloon anymore on modern Windows versions, but a regular notification.
        //!
        //! Long texts will be truncated. For more information on this and the parameters, see <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw>.
        //!
        //! Doesn't return an error, if the notification is suppressed.

        self.notify_icon_data.dwInfoFlags.0 |= match icon {
            BalloonIcon::None => NIIF_NONE.0,
            BalloonIcon::Info => NIIF_INFO.0,
            BalloonIcon::Warning => NIIF_WARNING.0,
            BalloonIcon::Error => NIIF_ERROR.0,
            BalloonIcon::User => NIIF_USER.0,
        };

        if let Some(title) = title {
            let title: HSTRING = title.into();
            title.write_truncated(&mut self.notify_icon_data.szInfoTitle);

            // Note: The docs (as of Dec. 2023) say: "If the szInfoTitle member is zero-length, the icon is not shown." On Windows 10, this isn't true. But the OS trims the string, and then, if it's empty, displays the message text in title style. This can extend this function's `Option` semantics, but the user should either not notice this behavior or not find it objectionable.
        } else {
            self.notify_icon_data.szInfoTitle[0] = 0;
        }

        let text: HSTRING = text.into();
        if text.is_empty() {
            const SPACE_WIDE_STR: [u16; 2] = [' ' as _, 0];

            // Prevent balloon staying hidden or hiding. (As of Dec. 2023, the OS trims this value, but judges emptiness before trimming.)
            self.notify_icon_data.szInfo[..SPACE_WIDE_STR.len()].copy_from_slice(&SPACE_WIDE_STR);
        } else {
            text.write_truncated(&mut self.notify_icon_data.szInfo);
        }

        if realtime_only {
            self.notify_icon_data.uFlags |= NIF_REALTIME;
        }
        if !override_quiet_time {
            self.notify_icon_data.dwInfoFlags.0 |= NIIF_RESPECT_QUIET_TIME.0;
        }
        if !allow_sound {
            self.notify_icon_data.dwInfoFlags.0 |= NIIF_NOSOUND.0;
        }

        let result = self.call_modify();

        self.notify_icon_data.szInfo[0] = 0; // Prevent new notification on other change.
        self.notify_icon_data.uFlags &= !NIF_REALTIME;
        self.notify_icon_data.dwInfoFlags.0 &= !(NIIF_NONE.0
            | NIIF_INFO.0
            | NIIF_WARNING.0
            | NIIF_ERROR.0
            | NIIF_USER.0
            | NIIF_RESPECT_QUIET_TIME.0
            | NIIF_NOSOUND.0);

        result
    }

    pub fn hide_balloon(&mut self) -> windows::core::Result<()> {
        self.notify_icon_data.szInfo[0] = 0;
        self.call_modify()
    }

    pub fn simplifying_translate_window_msg(
        &mut self,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> SimplifiedTrayIconMsg {
        let msg = translate_window_msg(wparam, lparam);

        match msg.msg_id as _ {
            NIN_SELECT | NIN_KEYSELECT => {
                // NIN_SELECT - After every up-event of the primary mouse button.
                // NIN_KEYSELECT - Once on Space, twice on Enter (when not holding the key; as of Dec. 2023).
                //
                // Since Space and Enter key presses can't be distinguished, and an Enter key press sends two undistinguishable events, the logic of reacting only once on double-click is also applied to the keyboard events.

                if self.last_activation_time.elapsed().as_millis()
                    > unsafe { GetDoubleClickTime() } as _
                {
                    self.last_activation_time = Instant::now();
                    SimplifiedTrayIconMsg::Activated
                } else {
                    SimplifiedTrayIconMsg::Other(msg)
                }
            }

            // Context menu request via mouse or keyboard.
            WM_CONTEXTMENU => SimplifiedTrayIconMsg::ContextMenuRequested { x: msg.x, y: msg.y },

            _ => SimplifiedTrayIconMsg::Other(msg),
        }
    }

    pub fn delete(&mut self) -> windows::core::Result<()> {
        //! Removes the icon from the tray, making this instance unusable for further actions.
        //!
        //! Should be used before destroying the associated window.

        unsafe { Shell_NotifyIconW(NIM_DELETE, &self.notify_icon_data).ok_or_e_fail() }
    }

    fn call_modify(&mut self) -> windows::core::Result<()> {
        unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.notify_icon_data).ok_or_e_fail() }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        // Calling this on an already deleted icon is simply a no-op.
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.notify_icon_data);
        }
    }
}

pub enum BalloonIcon {
    None,
    Info,
    Warning,
    Error,
    /// The icon set with [`TrayIcon::set_balloon_icon()`].
    User,
}

pub enum SimplifiedTrayIconMsg {
    /// Tray icon was clicked or double-clicked with primary mouse button, or Space or Enter was pressed on a keyboard-focused icon.
    ///
    /// Repeating the action in the double-click time frame leads to an `Other` event instead, which should be ignored (because only *some* occurrences of the respective message IDs are available, while others are transformed).
    Activated,
    /// Secondary mouse button was pressed, or context menu key/Shift+F10 was pressed on a keyboard-focused icon. With x-and-y virtual-screen coordinates.
    ContextMenuRequested {
        x: i16,
        y: i16,
    },
    Other(TrayIconMsg),
}

pub fn translate_window_msg(wparam: WPARAM, lparam: LPARAM) -> TrayIconMsg {
    TrayIconMsg {
        msg_id: lparam.low_u16() as _, // `u32` makes comparisons nicer.
        icon_id: lparam.high_u16(),
        x: wparam.low_i16(),
        y: wparam.high_i16(),
    }
}

pub struct TrayIconMsg {
    msg_id: u32,
    icon_id: u16,
    x: i16,
    y: i16,
}

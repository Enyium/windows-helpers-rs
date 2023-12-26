use crate::{
    bit_manipulation::Width32BitPortion, foundation::LParamExt, windows,
    wnds_and_msging::TimerProcExt,
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PBT_POWERSETTINGCHANGE, TIMERPROC},
};

pub fn translate_command_msg(wparam: WPARAM, lparam: LPARAM) -> CommandMsg {
    match wparam.high_u16() {
        0 => CommandMsg::MenuItem {
            id: wparam.low_u16(),
        },
        1 => CommandMsg::Accelerator {
            id: wparam.low_u16(),
        },
        wparam_high_u16 => CommandMsg::ControlMsg {
            msg_id: wparam_high_u16,
            control_id: wparam.low_u16(),
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

/// Activate feature `windows_<version>_f_Win32_System_Power`.
#[cfg(feature = "f_Win32_System_Power")]
pub unsafe fn translate_power_broadcast_msg(wparam: WPARAM, lparam: &LPARAM) -> PowerBroadcastMsg {
    if wparam.0 == PBT_POWERSETTINGCHANGE as _ {
        PowerBroadcastMsg::PowerSettingChange {
            setting: lparam.cast_to_ref(),
        }
    } else {
        PowerBroadcastMsg::Other {
            event: wparam.0 as _,
        }
    }
}

/// Activate feature `windows_<version>_f_Win32_System_Power`.
#[cfg(feature = "f_Win32_System_Power")]
pub enum PowerBroadcastMsg<'a> {
    PowerSettingChange {
        setting: &'a windows::Win32::System::Power::POWERBROADCAST_SETTING,
    },
    Other {
        event: u32,
    },
}

pub unsafe fn translate_timer_msg(wparam: WPARAM, lparam: LPARAM) -> TimerMsg {
    TimerMsg {
        timer_id: wparam.0,
        callback: TIMERPROC::from_lparam(lparam),
    }
}

pub struct TimerMsg {
    pub timer_id: usize,
    pub callback: TIMERPROC,
}

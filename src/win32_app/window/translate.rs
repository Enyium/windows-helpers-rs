use crate::{bit_manipulation::Width32BitPortion, windows, wnds_and_msging::TimerProcExt};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::TIMERPROC,
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

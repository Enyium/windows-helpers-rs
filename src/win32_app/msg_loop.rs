//! Functions to run a blocking Win32 message loop with `GetMessageW()` etc. Necessary for window procedures, timer callbacks and hook callbacks.

use crate::windows;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_QUIT},
};

use crate::error::ResultExt;

pub fn run() -> windows::core::Result<usize> {
    //! Runs a message loop, ignoring custom thread messages.
    //!
    //! If successful, returns the exit code received via `WM_QUIT` from `PostQuitMessage()` that the process should return. If unsuccessful and you can handle the error, the function can be rerun in a loop.

    loop {
        let msg = run_till_thread_msg()?;
        if msg.message == WM_QUIT {
            break Ok(msg.wParam.0);
        }
    }
}

pub fn run_till_thread_msg() -> windows::core::Result<MSG> {
    //! Runs a message loop until a thread message is received.
    //!
    //! In most programs, the only thread message will be `WM_QUIT` (sent via `PostQuitMessage()`). But others are possible via `PostThreadMessageW()` and `PostMessageW()`.

    let mut msg = MSG::default();

    loop {
        // (`GetMessageW()` calls hook callbacks without returning.)
        match unsafe { GetMessageW(&mut msg, HWND(0), 0, 0).0 } {
            -1 => break Result::err_from_win32(),

            // Received `WM_QUIT` thread message. Caller must check `msg.message` against `WM_QUIT`.
            // (`GetMessageW()` return value is checked instead of treating `WM_QUIT` like all thread messages, in case abusive behavior caused `msg.hwnd` to be non-zero, which is possible via `PostMessageW()`.)
            0 => break Ok(msg),

            _ => {
                // Propagate window message to window procedure.
                // As confirmed by a test, `DispatchMessageW()` also calls the timer callback on `WM_TIMER` when `msg.hwnd` is 0. Example code also does it this way. (https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues) So, the calls are just made for all thread messages. Custom thread messages are ignored by them. (Docs: "DispatchMessage will call the TimerProc callback function specified in the call to the SetTimer function used to install the timer." [https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-timer])
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // Return thread message.
                if msg.hwnd.0 == 0 {
                    break Ok(msg);
                }
            }
        }
    }
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use crate::windows;
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{PostQuitMessage, SetTimer},
    };

    #[ignore]
    #[test]
    fn set_timer() -> windows::core::Result<()> {
        extern "system" fn on_timer(hwnd: HWND, msg_id: u32, event_id: usize, time: u32) {
            println!("timer event: {hwnd:?}, 0x{msg_id:x?}, {event_id:?}, {time:?}");
            unsafe { PostQuitMessage(0) };
        }

        unsafe {
            SetTimer(HWND(0), 0, 500 /*ms*/, Some(on_timer))
        };
        super::run()?;
        println!("after msg loop");

        Ok(())
    }
}

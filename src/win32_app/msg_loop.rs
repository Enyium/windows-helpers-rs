//! Functions to run a blocking Win32 message loop with [`GetMessageW()`][1] etc. Necessary for window procedures, hook callbacks, timer callbacks and more.
//!
//! [1]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew

use crate::{core::ResultExt, windows, Null};
use std::cell::Cell;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG, WM_QUIT,
    },
};

thread_local! {
    /// The exit code set with [`quit_now()`]. Same data type as with [`PostQuitMessage()`][1].
    ///
    /// [1]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage
    static QUIT_NOW_EXIT_CODE: Cell<Option<i32>> = const { Cell::new(None) };
}

pub fn run() -> windows::core::Result<usize> {
    //! Runs a message loop, ignoring custom thread messages.
    //!
    //! If successful, returns the exit code received via [`WM_QUIT`][1] from [`PostQuitMessage()`][2] that the process should return. If unsuccessful and you can handle the error, the function can be rerun in a loop.
    //!
    //! [1]: https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-quit
    //! [2]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage

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
    //! In most programs, the only thread message will be [`WM_QUIT`][1] (sent via [`PostQuitMessage()`][1]). But others are possible via [`PostThreadMessageW()`][3] and [`PostMessageW()`][4].
    //!
    //! [1]: https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-quit
    //! [2]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage
    //! [3]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew
    //! [4]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postmessagew

    let mut msg = MSG::default();

    loop {
        // (`GetMessageW()` calls hook callbacks without returning.)
        let mut get_msg_retval = unsafe { GetMessageW(&mut msg, HWND::NULL, 0, 0).0 };

        if get_msg_retval == -1 {
            break Result::err_from_win32();
        } else {
            if let Some(exit_code) = QUIT_NOW_EXIT_CODE.get() {
                get_msg_retval = 0;
                msg.hwnd = HWND::NULL;
                msg.message = WM_QUIT;
                msg.wParam = WPARAM(exit_code as _);
                msg.lParam = LPARAM(0);
            }

            if get_msg_retval == 0 {
                // Received `WM_QUIT` thread message. Caller must check `msg.message` against `WM_QUIT`.
                // (`GetMessageW()` return value is checked instead of treating `WM_QUIT` like all thread messages, in case abusive behavior caused `msg.hwnd` to be non-zero, which is possible via `PostMessageW()`.)
                break Ok(msg);
            } else {
                // Propagate window message to window procedure.
                // As confirmed by a test, `DispatchMessageW()` also calls the timer callback on `WM_TIMER` when `msg.hwnd` is 0. Official example code also does it this way. (https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues) So, the calls are just made for all thread messages. Custom thread messages are ignored by them. (Docs: "DispatchMessage will call the TimerProc callback function specified in the call to the SetTimer function used to install the timer." [https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-timer])
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // Return thread message.
                if msg.hwnd.is_null() {
                    break Ok(msg);
                }
            }
        }
    }
}

pub fn quit_now(exit_code: i32) {
    //! Causes the message loop to quit as soon as possible.
    //!
    //! Can be used in case of exceptional errors. Note that this function doesn't have the never return type (`!`).
    //!
    //! The function saves the exit code in thread-local storage and posts a message. The very next message that the message loop retrieves will then be changed to a `WM_QUIT` message with that exit code, which causes the loop to return.

    QUIT_NOW_EXIT_CODE.set(Some(exit_code));
    unsafe { PostQuitMessage(exit_code) };
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use crate::{windows, Null};
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
            SetTimer(HWND::NULL, 0, 500 /*ms*/, Some(on_timer))
        };
        super::run()?;
        println!("after msg loop");

        Ok(())
    }
}

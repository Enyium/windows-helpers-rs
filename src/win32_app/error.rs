use super::msg_loop;
use crate::windows;
use std::cell::RefCell;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;

thread_local! {
    static APP_ERROR: RefCell<Option<Box<dyn std::error::Error + Send + Sync>>> = RefCell::new(None);
}

pub fn set_app_error_if_absent<E>(error: E)
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    //! Sets a thread-local error that can be retrieved with [`take_app_error()`], if one wasn't set already.

    APP_ERROR.with_borrow_mut(|app_error| {
        if app_error.is_none() {
            *app_error = Some(error.into());
        }
    });
}

pub fn clear_app_error() {
    APP_ERROR.with_borrow_mut(|app_error| {
        *app_error = None;
    });
}

pub fn take_app_error() -> Option<Box<dyn std::error::Error + Send + Sync>> {
    //! Clears the app error and returns it.

    APP_ERROR.with_borrow_mut(|app_error| app_error.take())
}

// pub fn take_app_error_or<E>(error: E) -> Box<dyn std::error::Error + Send + Sync>
// where
//     E: Into<Box<dyn std::error::Error + Send + Sync>>,
// {
//     //! Clears the app error and returns it, or, if not present, the provided error.
//
//     take_app_error().unwrap_or(error.into())
// }

pub fn just_try<F, T, E>(action: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    //! Just returns the `Result`, so you can easily gather `?` uses.

    action()
}

pub fn try_or_set_app_error<F, T, E>(action: F) -> Option<T>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    //! Calls [`set_app_error_if_absent()`] on `Err`. Returns the `Ok` value in `Some`.

    match action() {
        Ok(t) => Some(t),
        Err(error) => {
            set_app_error_if_absent(error);
            None
        }
    }
}

pub fn try_or_post_quit<F, T, E>(action: F) -> Option<T>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    //! Calls [`set_app_error_if_absent()`] and [`PostQuitMessage(1)`][1] on `Err`. Returns the `Ok` value in `Some`.
    //!
    //! [1]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage

    let option = try_or_set_app_error(action);
    if option.is_none() {
        unsafe { PostQuitMessage(1) };
    }
    option
}

pub fn try_or_quit_now<F, T, E>(action: F) -> Option<T>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    //! Calls [`set_app_error_if_absent()`] and <code>[super::msg_loop::quit_now]\(1\)</code> on `Err`. Returns the `Ok` value in `Some`.

    let option = try_or_set_app_error(action);
    if option.is_none() {
        msg_loop::quit_now(1);
    }
    option
}

pub fn try_or_panic<F, T, E>(action: F) -> T
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    //! Panics on `Err`, with debug-stringified payload. Returns the `Ok` value.

    action().unwrap_or_else(|e| {
        let error: Box<dyn std::error::Error> = e.into();
        panic!("{:?}", error);
    })
}

pub fn try_then_favor_app_error<F, T, E>(
    action: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    //! After running the action, returns the error from [`take_app_error()`], or, if not present, the action's `Result`.
    //!
    //! Can be used with fundamental actions including running the message loop. Even when the actions don't fail, there might still be an app error.

    let result = action();
    if let Some(error) = take_app_error() {
        Err(error)
    } else {
        result.map_err(|e| e.into())
    }
}

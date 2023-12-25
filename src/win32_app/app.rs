use super::window::{Window, WindowClass};
use crate::{cell::ReentrantRefCell, windows};
use std::rc::Rc;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

pub struct InvisibleWindowAppHelper<'a> {
    _window: Window,
    _window_class: WindowClass<'a>,
}

impl<'a> InvisibleWindowAppHelper<'a> {
    pub unsafe fn make_app<App>() -> windows::core::Result<(Self, Rc<ReentrantRefCell<Option<App>>>)>
    where
        App: AppLike<Self> + 'a,
    {
        //! Bootstraps an app with simple message-receiving capabilities.
        //!
        //! Drop the first return value (the helper) last. This is ensured by a regular binding `let (_app_helper, _app) = ...` when not passing the last return value out of the scope.
        //!
        //! # Safety
        //! See [`AppLike::wnd_proc()`].

        let app = Rc::new(ReentrantRefCell::new(None::<App>));
        let weak_app = Rc::downgrade(&app);

        let window_class = WindowClass::new(move |hwnd, msg_id, wparam, lparam| {
            // (`Weak` is necessary to prevent a circular dependency, which would prevent the `Drop` impl from being called.)
            weak_app.upgrade().and_then(|app_cell| unsafe {
                app_cell.borrow_mut_reentrant(|optional_app| match optional_app {
                    None => {
                        let (new_app, lresult) =
                            App::startup_wnd_proc(hwnd, msg_id, wparam, lparam);
                        *optional_app = new_app;
                        lresult
                    }
                    Some(app) => app.wnd_proc(hwnd, msg_id, wparam, lparam),
                })
            })
        })?;

        let window = Window::new_invisible(&window_class)?;

        let helper = Self {
            _window_class: window_class,
            _window: window,
        };

        Ok((helper, app))
    }
}

pub trait AppLike<Helper>
where
    Self: Sized,
{
    /// Where you let the helper make your app.
    fn new() -> windows::core::Result<(Helper, Rc<ReentrantRefCell<Option<Self>>>)>;

    /// The window procedure called initially, until you provide an instance through the first return value. The second return value is as in [`Self::wnd_proc()`]. You can, e.g., create the instance on `WM_CREATE`.
    fn startup_wnd_proc(
        hwnd: HWND,
        msg_id: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> (Option<Self>, Option<LRESULT>);

    /// The regular window procedure called when [`Self::startup_wnd_proc()`] isn't called anymore.
    ///
    /// See also [`window::WindowClass`].
    ///
    /// # Safety
    /// You must use [`Self::reenter_wnd_proc()`] when appropriate.
    fn wnd_proc(
        &mut self,
        hwnd: HWND,
        msg_id: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT>;

    /// A helper function that simply takes the same `self` parameter as [`Self::wnd_proc()`] to cause compiler errors, if necessary, when functions are called that synchronously call the window procedure and thus borrow `&mut self` again (via `ReentrantRefCell`). Anything other than simple reborrowing is against the rules. This prevents multiple simultaneous borrows.
    ///
    /// The function can be viewed as adding a `self` parameter to Windows API functions, as if they would belong to the type.
    ///
    /// See [`ReentrantRefCell::borrow_mut_reentrant()`] for more information.
    fn reenter_wnd_proc<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        f(self)
    }
}

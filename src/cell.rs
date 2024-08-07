use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    panic::{self, AssertUnwindSafe},
};

/// A `RefCell` that allows to recursively retrieve a mutable reference.
///
/// Like [`std::cell::RefCell`], but with an additional [`Self::borrow_mut_reentrant()`] method. (If needed, the type could call through to more of `RefCell`'s other methods.)
pub struct ReentrantRefCell<T: ?Sized> {
    num_mut_re_borrows: Cell<usize>,
    // `RefCell` not implementing `Sync` will make the struct not implement it either.
    ref_cell: RefCell<T>,
}

// Like for `RefCell`.
unsafe impl<T: ?Sized> Send for ReentrantRefCell<T> where T: Send {}

impl<T> ReentrantRefCell<T> {
    pub fn new(data: T) -> Self {
        ReentrantRefCell {
            num_mut_re_borrows: Cell::new(0),
            ref_cell: RefCell::new(data),
        }
    }

    pub fn borrow(&self) -> Ref<T> {
        #![inline]

        self.ref_cell.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        #![inline]

        self.ref_cell.borrow_mut()
    }

    pub unsafe fn borrow_mut_reentrant<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        //!  Mutably borrows the wrapped value for the duration of the closure call, again allowing mutable borrows by means of this method inside of the closure and the functions it calls.
        //!
        //! It borrows with [`std::cell::RefCell::borrow_mut()`] for the outermost call, which means additional attempts to borrow during the outermost borrow, other than by means of this method, will panic. Repeated inner calls provide the mutable reference that the outermost call made available.
        //!
        //! The function is useful when dealing with an FFI and foreign code calls into your callback with a [`ReentrantRefCell`] at hand, you then call an FFI function and, during this call, the foreign code calls into your callback again. This happens, e.g., with [window procedures][1] on Windows when calling functions like [`DestroyWindow()`][2] or [`MoveWindow()`][3] in the procedure itself.
        //!
        //! # Safety
        //! You are responsible to only call reentrance causing functions (like FFI functions) as if they had a `&mut self` parameter and wouldn't cause a compiler error with that. I.e., you must, e.g., not borrow something mutably from the mutable reference you get, call the reentrance causing function and then continue to use the borrow from before. When used in the relevant cases, a helper function that simply demands a `&mut self` parameter and just calls through to the closure from its second parameter would desirably trigger compiler errors. It's unknown whether using such a helper function is necessary with regard to possible compiler optimizations when not using it.
        //!
        //! Searching for "sen" ("send"/"sent") on Windows API function doc pages seems to be a good way to check whether a function may synchronously call the window procedure.
        //!
        //! [1]: https://learn.microsoft.com/en-us/windows/win32/winmsg/window-procedures
        //! [2]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow
        //! [3]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-movewindow

        let prev_num_mut_re_borrows = self
            .num_mut_re_borrows
            .replace(self.num_mut_re_borrows.get() + 1);

        // Because a caught panic is resumed below, unwind safety can be asserted. (A panic in the middle of mutating the data or whatever the closure closes over may leave some data broken. But since the panic is resumed, it's as if the panic wasn't caught.)
        let f_retval = panic::catch_unwind(AssertUnwindSafe(|| {
            if prev_num_mut_re_borrows == 0 {
                f(&mut *self.ref_cell.borrow_mut())
            } else {
                f(&mut *self.ref_cell.as_ptr())
            }
        }));

        self.num_mut_re_borrows.replace(prev_num_mut_re_borrows);

        f_retval.unwrap_or_else(|panic_payload| panic::resume_unwind(panic_payload))
    }
}

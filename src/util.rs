#![doc(hidden)]

use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    panic::{self, AssertUnwindSafe},
};

/// (DO NOT USE FROM OTHER CRATES: TODO: Offer type to crate <https://github.com/Amanieu/parking_lot>. [`catch_unwind()` makes `core` instead of `std` impossible. But the crate already uses `std`. Other `RefCell` methods can be called through to also.])
///
/// A `RefCell` that allows to recursively retrieve a mutable reference.
///
/// Identical to [`std::cell::RefCell`], but with an additional [`borrow_mut_reentrant()`] method.
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

    #[inline]
    pub fn borrow(&self) -> Ref<T> {
        self.ref_cell.borrow()
    }

    #[inline]
    pub fn borrow_mut(&self) -> RefMut<T> {
        self.ref_cell.borrow_mut()
    }

    ///  Mutably borrows the wrapped value for the duration of the closure call, again allowing mutable borrows by means of this method inside of the closure and the functions it calls.
    ///
    /// It borrows with [`std::cell::RefCell::borrow_mut()`] for the outermost call, which means additional attempts to borrow during the outermost borrow, other than by means of this method, will panic. Repeated inner calls provide the mutable reference that the outermost call made available.
    ///
    /// The function shines when dealing with an FFI and foreign code calls into your callback with a [`ReentrantRefCell`] at hand, you then call an FFI function and, during this call, the foreign code calls into your callback again. This happens, e.g., with [window procedures][1] on Windows when calling functions like [`DestroyWindow()`][2] or [`TrackPopupMenu()`][3] in the procedure itself.
    ///
    /// [1]: https://learn.microsoft.com/en-us/windows/win32/winmsg/window-procedures
    /// [2]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow
    /// [3]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-trackpopupmenu
    pub fn borrow_mut_reentrant<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        let prev_num_mut_re_borrows = self
            .num_mut_re_borrows
            .replace(self.num_mut_re_borrows.get() + 1);

        // Because a caught panic is resumed below, unwind safety can be asserted. (A panic in the middle of mutating the data or whatever the closure closes over may leave some data broken. But since the panic is resumed, it's as if the panic wasn't caught.)
        let f_retval = panic::catch_unwind(AssertUnwindSafe(|| {
            if prev_num_mut_re_borrows == 0 {
                f(&mut *self.ref_cell.borrow_mut())
            } else {
                f(unsafe { &mut *self.ref_cell.as_ptr() })
            }
        }));

        self.num_mut_re_borrows.replace(prev_num_mut_re_borrows);

        f_retval.unwrap_or_else(|panic_payload| panic::resume_unwind(panic_payload))
    }
}

use crate::{windows, Null};
use std::ops::Deref;

/// Holds a resource and a free-function (like a non-capturing closure) that is called when the guard is dropped.
///
/// Allows to couple resource acquisition and freeing, while treating the guard as the contained resource and ensuring freeing will happen. When writing the code, it's also nice to transfer the documentation into everything that has to happen in one go without having to split it into upper and lower or here- and there-code. In a function, Rust's drop order should ensure that later aquired resources are freed first.
///
/// For functions ending in Windows API function names (differently cased, like `..._destroy_icon()`), you have to activate crate features. First, see the repository's read-me. Then, derive the needed features from the Windows API function and the handle type the instance manages.
pub struct ResGuard<R: Copy> {
    resource: R,
    free_fn: fn(R),
}

impl<R: Copy> ResGuard<R> {
    pub fn new(resource: R, free: fn(R)) -> Self {
        //! Should normally not be needed.

        Self {
            resource: resource,
            free_fn: free,
        }
    }

    pub fn with_acquisition<A, E>(acquire: A, free: fn(R)) -> Result<Self, E>
    where
        A: FnOnce() -> Result<R, E>,
    {
        //! For use with functions that return the resource.

        Ok(Self {
            resource: acquire()?,
            free_fn: free,
        })
    }

    pub fn with_mut_acquisition<A, T, E>(acquire: A, free: fn(R)) -> Result<Self, E>
    where
        R: Null,
        A: FnOnce(&mut R) -> Result<T, E>,
    {
        //! For use with functions that provide the resource by means of an out-parameter.

        let mut resource = R::NULL;
        acquire(&mut resource)?;

        Ok(Self {
            resource,
            free_fn: free,
        })
    }

    pub fn two_with_mut_acquisition<A, T, E>(
        acquire_both: A,
        free_first: fn(R),
        free_second: fn(R),
    ) -> Result<(Self, Self), E>
    where
        R: Null,
        A: FnOnce(&mut R, &mut R) -> Result<T, E>,
    {
        //! For purpose, see [`Self::two_with_mut_acq_and_close_handle()`].

        let mut first_resource = R::NULL;
        let mut second_resource = R::NULL;
        acquire_both(&mut first_resource, &mut second_resource)?;

        Ok((
            Self {
                resource: first_resource,
                free_fn: free_first,
            },
            Self {
                resource: second_resource,
                free_fn: free_second,
            },
        ))
    }
}

macro_rules! impl_with_acq_and_free_fn {
    ($type:ty, $with_res:ident, $with_acq:ident, $with_acq_mut:ident, $free_fn:expr) => {
        impl ResGuard<$type> {
            const FREE_FN: fn($type) = $free_fn;

            pub fn $with_res(resource: $type) -> Self {
                Self::new(resource, Self::FREE_FN)
            }

            pub fn $with_acq<A, E>(acquire: A) -> Result<Self, E>
            where
                A: FnOnce() -> Result<$type, E>,
            {
                Self::with_acquisition(acquire, Self::FREE_FN)
            }

            pub fn $with_acq_mut<A, T, E>(acquire: A) -> Result<Self, E>
            where
                A: FnOnce(&mut $type) -> Result<T, E>,
            {
                Self::with_mut_acquisition(acquire, Self::FREE_FN)
            }
        }
    };
}

#[cfg(all(feature = "f_Win32_Foundation"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Foundation::HANDLE,
    with_res_and_close_handle,
    with_acq_and_close_handle,
    with_mut_acq_and_close_handle,
    |handle| {
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(handle) };
    }
);

#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HBITMAP,
    with_res_and_delete_object,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_bitmap| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_bitmap) };
    }
);

#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HBRUSH,
    with_res_and_delete_object,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_brush| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_brush) };
    }
);

#[cfg(feature = "windows_v0_48")]
#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::CreatedHDC,
    with_res_and_delete_dc,
    with_acq_and_delete_dc,
    with_mut_acq_and_delete_dc,
    |h_dc| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteDC(h_dc) };
    }
);

#[cfg(not(feature = "windows_v0_48"))]
#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HDC,
    with_res_and_delete_dc,
    with_acq_and_delete_dc,
    with_mut_acq_and_delete_dc,
    |h_dc| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteDC(h_dc) };
    }
);

#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HFONT,
    with_res_and_delete_object,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_font| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_font) };
    }
);

#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HGDIOBJ,
    with_res_and_delete_object,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_gdi_obj| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_gdi_obj) };
    }
);

#[cfg(feature = "windows_v0_48")]
#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_System_Memory"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Foundation::HGLOBAL,
    with_res_and_global_free,
    with_acq_and_global_free,
    with_mut_acq_and_global_free,
    |h_global| {
        let _ = unsafe { windows::Win32::System::Memory::GlobalFree(h_global) };
    }
);

#[cfg(not(feature = "windows_v0_48"))]
#[cfg(all(feature = "f_Win32_Foundation"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Foundation::HGLOBAL,
    with_res_and_global_free,
    with_acq_and_global_free,
    with_mut_acq_and_global_free,
    |h_global| {
        let _ = unsafe { windows::Win32::Foundation::GlobalFree(h_global) };
    }
);

#[cfg(all(
    feature = "f_Win32_Foundation",
    feature = "f_Win32_UI_WindowsAndMessaging"
))]
impl_with_acq_and_free_fn!(
    windows::Win32::UI::WindowsAndMessaging::HICON,
    with_res_and_destroy_icon,
    with_acq_and_destroy_icon,
    with_mut_acq_and_destroy_icon,
    |h_icon| {
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyIcon(h_icon) };
    }
);

#[cfg(feature = "windows_v0_48")]
#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_System_Memory"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Foundation::HLOCAL,
    with_res_and_local_free,
    with_acq_and_local_free,
    with_mut_acq_and_local_free,
    |h_local| {
        let _ = unsafe { windows::Win32::System::Memory::LocalFree(h_local) };
    }
);

#[cfg(not(feature = "windows_v0_48"))]
#[cfg(all(feature = "f_Win32_Foundation"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Foundation::HLOCAL,
    with_res_and_local_free,
    with_acq_and_local_free,
    with_mut_acq_and_local_free,
    |h_local| {
        let _ = unsafe { windows::Win32::Foundation::LocalFree(h_local) };
    }
);

#[cfg(all(
    feature = "f_Win32_Foundation",
    feature = "f_Win32_UI_WindowsAndMessaging"
))]
impl_with_acq_and_free_fn!(
    windows::Win32::UI::WindowsAndMessaging::HMENU,
    with_res_and_destroy_menu,
    with_acq_and_destroy_menu,
    with_mut_acq_and_destroy_menu,
    |h_menu| {
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyMenu(h_menu) };
    }
);

#[cfg(feature = "windows_v0_48")]
#[cfg(all(
    feature = "f_Win32_Foundation",
    feature = "f_Win32_System_LibraryLoader"
))]
impl_with_acq_and_free_fn!(
    windows::Win32::Foundation::HMODULE,
    with_res_and_free_library,
    with_acq_and_free_library,
    with_mut_acq_and_free_library,
    |h_module| {
        let _ = unsafe { windows::Win32::System::LibraryLoader::FreeLibrary(h_module) };
    }
);

#[cfg(not(feature = "windows_v0_48"))]
#[cfg(all(feature = "f_Win32_Foundation"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Foundation::HMODULE,
    with_res_and_free_library,
    with_acq_and_free_library,
    with_mut_acq_and_free_library,
    |h_module| {
        let _ = unsafe { windows::Win32::Foundation::FreeLibrary(h_module) };
    }
);

#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HPALETTE,
    with_res_and_delete_object,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_palette| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_palette) };
    }
);

#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HPEN,
    with_res_and_delete_object,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_pen| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_pen) };
    }
);

#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_Graphics_Gdi"))]
impl_with_acq_and_free_fn!(
    windows::Win32::Graphics::Gdi::HRGN,
    with_res_and_delete_object,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_rgn| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_rgn) };
    }
);

#[cfg(feature = "windows_v0_48")]
#[cfg(all(feature = "f_Win32_Foundation", feature = "f_Win32_System_Memory"))]
impl_with_acq_and_free_fn!(
    windows::core::PWSTR,
    with_res_and_local_free,
    with_acq_and_local_free,
    with_mut_acq_and_local_free,
    |pwstr| {
        let _ = unsafe {
            windows::Win32::System::Memory::LocalFree(windows::Win32::Foundation::HLOCAL(
                pwstr.0 as _,
            ))
        };
    }
);

//. Useful for functions like `ConvertSidToStringSidW()` and `FormatMessageW()`, which allocate for you and are documented to require a call to `LocalFree()`.
#[cfg(not(feature = "windows_v0_48"))]
#[cfg(all(feature = "f_Win32_Foundation"))]
impl_with_acq_and_free_fn!(
    windows::core::PWSTR,
    with_res_and_local_free,
    with_acq_and_local_free,
    with_mut_acq_and_local_free,
    |pwstr| {
        let _ = unsafe {
            windows::Win32::Foundation::LocalFree(windows::Win32::Foundation::HLOCAL(
                pwstr.0.cast(),
            ))
        };
    }
);

#[cfg(feature = "f_Win32_Foundation")]
impl ResGuard<windows::Win32::Foundation::HANDLE> {
    // (`FREE_FN` was already defined in previous impl with this type parameter.)

    pub fn two_with_mut_acq_and_close_handle<A, T, E>(acquire_both: A) -> Result<(Self, Self), E>
    where
        A: FnOnce(
            &mut windows::Win32::Foundation::HANDLE,
            &mut windows::Win32::Foundation::HANDLE,
        ) -> Result<T, E>,
    {
        //! For a function like `CreatePipe()` that returns two resources at once.

        Self::two_with_mut_acquisition(acquire_both, Self::FREE_FN, Self::FREE_FN)
    }
}

impl<R: Copy> Deref for ResGuard<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

impl<R: Copy> Drop for ResGuard<R> {
    fn drop(&mut self) {
        (self.free_fn)(self.resource);
    }
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use super::ResGuard;
    use crate::{
        core::{CheckNullError, CheckNumberError},
        windows, Null,
    };
    use std::{mem, ptr};
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{CloseHandle, COLORREF},
            Graphics::Gdi::{CreateSolidBrush, GetObjectW, HBRUSH, LOGBRUSH},
            Storage::FileSystem::{ReadFile, WriteFile},
            System::{
                Pipes::CreatePipe,
                Threading::{CreateEventW, SetEvent},
            },
        },
    };

    #[test]
    fn new() {
        let event_handle = unsafe { CreateEventW(None, true, false, PCWSTR::NULL) }
            .expect("should be able to create event handle");
        let event_handle = ResGuard::new(event_handle, |handle| {
            let _ = unsafe { CloseHandle(handle) };
        });

        assert_eq!(unsafe { SetEvent(*event_handle) }, Ok(()));
    }

    #[test]
    fn with_acq_and_close_handle() {
        let event_handle = ResGuard::with_acq_and_close_handle(|| unsafe {
            CreateEventW(None, true, false, PCWSTR::NULL)
        })
        .expect("should be able to create event handle");

        assert_eq!(unsafe { SetEvent(*event_handle) }, Ok(()));
    }

    #[test]
    fn two_with_mut_acq_and_close_handle() {
        // Acquire pipe handles.
        let (read_handle, write_handle) =
            ResGuard::two_with_mut_acq_and_close_handle(|read_handle, write_handle| unsafe {
                CreatePipe(read_handle, write_handle, None, 0)
            })
            .expect("should be able to create pipe handles");

        // Write.
        let bytes = [123, 45, 67];
        let mut bytes_written = 0;
        assert_eq!(
            unsafe { WriteFile(*write_handle, Some(&bytes), Some(&mut bytes_written), None,) },
            Ok(())
        );
        assert_eq!(bytes_written as usize, bytes.len());

        // Read.
        let mut buffer = Vec::new();
        buffer.resize(bytes.len(), 0);
        let mut bytes_read = 0;
        assert_eq!(
            unsafe { ReadFile(*read_handle, Some(&mut buffer), Some(&mut bytes_read), None) },
            Ok(())
        );
        assert_eq!(bytes_read as usize, buffer.len());
        assert_eq!(buffer, bytes);
    }

    #[test]
    fn with_acq_and_delete_object() -> windows::core::Result<()> {
        //! Tests handle type conversion: `HBRUSH` to `HGDIOBJ`.

        const BGR: u32 = 0x123456;

        let h_brush = ResGuard::<HBRUSH>::with_acq_and_delete_object(|| {
            unsafe { CreateSolidBrush(COLORREF(BGR)) }.nonnull_or_e_handle()
        })?;

        let mut log_brush = LOGBRUSH::default();
        unsafe {
            GetObjectW(
                *h_brush,
                mem::size_of::<LOGBRUSH>() as _,
                Some(ptr::addr_of_mut!(log_brush).cast()),
            )
        }
        .nonzero_or_e_fail()?;

        assert_eq!(log_brush.lbColor.0, BGR);

        Ok(())
    }
}

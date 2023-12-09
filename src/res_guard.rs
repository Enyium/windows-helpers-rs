use std::ops::Deref;

/// Holds a resource and a free-closure that is called when the guard is dropped.
///
/// Allows to couple resource acquisition and freeing, while treating the guard as the contained resource and ensuring freeing will happen. When writing the code, it's also nice to transfer the documentation into everything that has to happen in one go without having to split it into upper and lower or here- and there-code. In a function, Rust's drop order should ensure that later aquired resources are freed first.
pub struct ResGuard<R, F>
where
    F: FnOnce(R),
{
    resource: Option<R>,
    free: Option<F>,
}

impl<R, F> ResGuard<R, F>
where
    F: FnOnce(R),
{
    pub fn new(resource: R, free: F) -> Self {
        //! Should normally not be needed.

        Self {
            resource: Some(resource),
            free: Some(free),
        }
    }

    pub fn with_acquisition<A, E>(acquire: A, free: F) -> Result<Self, E>
    where
        A: FnOnce() -> Result<R, E>,
    {
        //! For functions that return the resource.

        Ok(Self {
            resource: Some(acquire()?),
            free: Some(free),
        })
    }

    pub fn with_mut_acquisition<A, T, E>(acquire: A, free: F) -> Result<Self, E>
    where
        A: FnOnce(&mut R) -> Result<T, E>,
        R: Default,
    {
        //! For functions that provide the resource by means of an out-parameter.

        let mut resource = R::default();
        acquire(&mut resource)?;

        Ok(Self {
            resource: Some(resource),
            free: Some(free),
        })
    }
}

macro_rules! impl_with_acq_and_star {
    ($feature:expr, $type:ty, $acq:ident, $acq_mut:ident, $free_fn:expr) => {
        #[cfg(feature = $feature)]
        impl ResGuard<$type, fn($type)> {
            #[doc = concat!("Activate feature `", $feature, "`.")]
            pub fn $acq<A, E>(acquire: A) -> Result<ResGuard<$type, fn($type)>, E>
            where
                A: FnOnce() -> Result<$type, E>,
            {
                Self::with_acquisition(acquire, $free_fn)
            }

            /// Activate same feature.
            pub fn $acq_mut<A, T, E>(acquire: A) -> Result<ResGuard<$type, fn($type)>, E>
            where
                A: FnOnce(&mut $type) -> Result<T, E>,
            {
                Self::with_mut_acquisition(acquire, $free_fn)
            }
        }
    };
}

// Note: The impls require features gating the use of all their inner types, because of the following experience: In v0.48, the `windows` crate had `GlobalFree()` in the module `windows::Win32::System::Memory`, but v0.52 in `windows::Win32::Foundation`. When the impl only had the `Win32_Foundation` feature and the user didn't need `GlobalFree()`, but another free-function also gated with `Win32_Foundation`, there would be an unnecessary error about an incorrectly located function.

impl_with_acq_and_star!(
    "HANDLE_CloseHandle",
    windows::Win32::Foundation::HANDLE,
    with_acq_and_close_handle,
    with_mut_acq_and_close_handle,
    |handle| {
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(handle) };
    }
);

impl_with_acq_and_star!(
    "HBITMAP_DeleteObject",
    windows::Win32::Graphics::Gdi::HBITMAP,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_bitmap| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_bitmap) };
    }
);

impl_with_acq_and_star!(
    "HBRUSH_DeleteObject",
    windows::Win32::Graphics::Gdi::HBRUSH,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_brush| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_brush) };
    }
);

impl_with_acq_and_star!(
    "HDC_DeleteDC",
    windows::Win32::Graphics::Gdi::HDC,
    with_acq_and_delete_dc,
    with_mut_acq_and_delete_dc,
    |h_dc| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteDC(h_dc) };
    }
);

impl_with_acq_and_star!(
    "HFONT_DeleteObject",
    windows::Win32::Graphics::Gdi::HFONT,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_font| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_font) };
    }
);

impl_with_acq_and_star!(
    "HGDIOBJ_DeleteObject",
    windows::Win32::Graphics::Gdi::HGDIOBJ,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_gdi_obj| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_gdi_obj) };
    }
);

impl_with_acq_and_star!(
    "HGLOBAL_GlobalFree",
    windows::Win32::Foundation::HGLOBAL,
    with_acq_and_global_free,
    with_mut_acq_and_global_free,
    |h_global| {
        let _ = unsafe { windows::Win32::Foundation::GlobalFree(h_global) };
    }
);

impl_with_acq_and_star!(
    "HICON_DestroyIcon",
    windows::Win32::UI::WindowsAndMessaging::HICON,
    with_acq_and_destroy_icon,
    with_mut_acq_and_destroy_icon,
    |h_icon| {
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyIcon(h_icon) };
    }
);

impl_with_acq_and_star!(
    "HLOCAL_LocalFree",
    windows::Win32::Foundation::HLOCAL,
    with_acq_and_local_free,
    with_mut_acq_and_local_free,
    |h_local| {
        let _ = unsafe { windows::Win32::Foundation::LocalFree(h_local) };
    }
);

impl_with_acq_and_star!(
    "HMENU_DestroyMenu",
    windows::Win32::UI::WindowsAndMessaging::HMENU,
    with_acq_and_destroy_menu,
    with_mut_acq_and_destroy_menu,
    |h_menu| {
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyMenu(h_menu) };
    }
);

impl_with_acq_and_star!(
    "HMODULE_FreeLibrary",
    windows::Win32::Foundation::HMODULE,
    with_acq_and_free_library,
    with_mut_acq_and_free_library,
    |h_module| {
        let _ = unsafe { windows::Win32::Foundation::FreeLibrary(h_module) };
    }
);

impl_with_acq_and_star!(
    "HPALETTE_DeleteObject",
    windows::Win32::Graphics::Gdi::HPALETTE,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_palette| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_palette) };
    }
);

impl_with_acq_and_star!(
    "HPEN_DeleteObject",
    windows::Win32::Graphics::Gdi::HPEN,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_pen| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_pen) };
    }
);

impl_with_acq_and_star!(
    "HRGN_DeleteObject",
    windows::Win32::Graphics::Gdi::HRGN,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_rgn| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_rgn) };
    }
);

impl<R, F> Deref for ResGuard<R, F>
where
    F: FnOnce(R),
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<R, F> Drop for ResGuard<R, F>
where
    F: FnOnce(R),
{
    fn drop(&mut self) {
        self.free.take().unwrap()(self.resource.take().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::CloseHandle,
            Storage::FileSystem::{ReadFile, WriteFile},
            System::{
                Pipes::CreatePipe,
                Threading::{CreateEventW, SetEvent},
            },
        },
    };

    use super::ResGuard;

    #[test]
    fn new() {
        let event_handle = unsafe { CreateEventW(None, true, false, PCWSTR::null()) }
            .expect("should be able to create event handle");
        let event_handle = ResGuard::new(event_handle, |handle| {
            let _ = unsafe { CloseHandle(handle) };
        });

        assert_eq!(unsafe { SetEvent(*event_handle) }, Ok(()));
    }

    #[test]
    #[cfg(feature = "HANDLE_CloseHandle")]
    fn with_acq_and_close_handle() {
        let event_handle = ResGuard::with_acq_and_close_handle(|| unsafe {
            CreateEventW(None, true, false, PCWSTR::null())
        })
        .expect("should be able to create event handle");

        assert_eq!(unsafe { SetEvent(*event_handle) }, Ok(()));
    }

    #[test]
    fn with_mut_acquisition() {
        // Acquire pipe handles.
        let pipe_handles = ResGuard::with_mut_acquisition(
            |(read_handle, write_handle)| unsafe { CreatePipe(read_handle, write_handle, None, 0) },
            |(read_handle, write_handle)| {
                let _ = unsafe { CloseHandle(read_handle) };
                let _ = unsafe { CloseHandle(write_handle) };
            },
        )
        .expect("should be able to create pipe handles");
        let (read_handle, write_handle) = *pipe_handles;

        // Write.
        let bytes = [123, 45, 67];
        let mut bytes_written = 0;
        assert_eq!(
            unsafe { WriteFile(write_handle, Some(&bytes), Some(&mut bytes_written), None,) },
            Ok(())
        );
        assert_eq!(bytes_written as usize, bytes.len());

        // Read.
        let mut buffer = Vec::new();
        buffer.resize(bytes.len(), 0);
        let mut bytes_read = 0;
        assert_eq!(
            unsafe { ReadFile(read_handle, Some(&mut buffer), Some(&mut bytes_read), None) },
            Ok(())
        );
        assert_eq!(bytes_read as usize, buffer.len());
        assert_eq!(buffer, bytes);
    }
}

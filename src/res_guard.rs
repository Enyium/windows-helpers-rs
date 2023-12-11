use crate::windows;
use std::ops::Deref;

/// Holds a resource and a free-closure that is called when the guard is dropped.
///
/// Allows to couple resource acquisition and freeing, while treating the guard as the contained resource and ensuring freeing will happen. When writing the code, it's also nice to transfer the documentation into everything that has to happen in one go without having to split it into upper and lower or here- and there-code. In a function, Rust's drop order should ensure that later aquired resources are freed first.
pub struct ResGuard<R, F>
where
    F: FnOnce(R),
{
    resource: Option<R>,
    free_fn: Option<F>,
}

impl<R, F> ResGuard<R, F>
where
    F: FnOnce(R),
{
    pub fn new(resource: R, free: F) -> Self {
        //! Should normally not be needed.

        Self {
            resource: Some(resource),
            free_fn: Some(free),
        }
    }

    pub fn with_acquisition<A, E>(acquire: A, free: F) -> Result<Self, E>
    where
        A: FnOnce() -> Result<R, E>,
    {
        //! For functions that return the resource.

        Ok(Self {
            resource: Some(acquire()?),
            free_fn: Some(free),
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
            free_fn: Some(free),
        })
    }

    pub fn two_with_mut_acquisition<A, T, E>(
        acquire_both: A,
        free_first: F,
        free_second: F,
    ) -> Result<(Self, Self), E>
    where
        A: FnOnce(&mut R, &mut R) -> Result<T, E>,
        R: Default,
    {
        //! For purpose, see [`Self::two_with_mut_acq_and_close_handle()`].

        let mut first_resource = R::default();
        let mut second_resource = R::default();
        acquire_both(&mut first_resource, &mut second_resource)?;

        Ok((
            Self {
                resource: Some(first_resource),
                free_fn: Some(free_first),
            },
            Self {
                resource: Some(second_resource),
                free_fn: Some(free_second),
            },
        ))
    }
}

macro_rules! impl_with_acq_and_free_fn {
    ($feature:literal, $type:ty, $acq:ident, $acq_mut:ident, $free_fn:expr) => {
        #[cfg(feature = $feature)]
        impl<R> ResGuard<R, fn(R)>
        where
            R: windows::core::CanInto<$type>
                + windows::core::TypeKind<TypeKind = windows::core::CopyType>
                + Clone,
        {
            pub fn $acq<A, E>(acquire: A) -> Result<ResGuard<R, fn(R)>, E>
            where
                A: FnOnce() -> Result<R, E>,
            {
                #![doc = concat!("Activate feature `windows_<version>_", $feature, "`.")]

                Self::with_acquisition(acquire, $free_fn)
            }

            pub fn $acq_mut<A, T, E>(acquire: A) -> Result<ResGuard<R, fn(R)>, E>
            where
                A: FnOnce(&mut R) -> Result<T, E>,
                R: Default,
            {
                #![doc = concat!("Activate feature `windows_<version>_", $feature, "`.")]

                Self::with_mut_acquisition(acquire, $free_fn)
            }
        }
    };
}

// Note: The impls require features gating the use of all their inner types, because of the following experience: In v0.48, the `windows` crate had `GlobalFree()` in the module `windows::Win32::System::Memory`, but v0.52 in `windows::Win32::Foundation`. When the impl only had the `Win32_Foundation` feature and the user didn't need `GlobalFree()`, but another free-function also gated with `Win32_Foundation`, there would be an unnecessary error about an incorrectly located function.

impl_with_acq_and_free_fn!(
    "HANDLE_CloseHandle",
    windows::Win32::Foundation::HANDLE,
    with_acq_and_close_handle,
    with_mut_acq_and_close_handle,
    |handle_compatible| {
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(handle_compatible) };
    }
);

#[cfg(feature = "windows_v0_48")]
impl_with_acq_and_free_fn!(
    "HDC_DeleteDC",
    windows::Win32::Graphics::Gdi::CreatedHDC,
    with_acq_and_delete_dc,
    with_mut_acq_and_delete_dc,
    |h_dc_compatible| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteDC(h_dc_compatible) };
    }
);

#[cfg(feature = "windows_v0_52")]
impl_with_acq_and_free_fn!(
    "HDC_DeleteDC",
    windows::Win32::Graphics::Gdi::HDC,
    with_acq_and_delete_dc,
    with_mut_acq_and_delete_dc,
    |h_dc_compatible| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteDC(h_dc_compatible) };
    }
);

impl_with_acq_and_free_fn!(
    "HGDIOBJ_DeleteObject",
    windows::Win32::Graphics::Gdi::HGDIOBJ,
    with_acq_and_delete_object,
    with_mut_acq_and_delete_object,
    |h_gdi_obj_compatible| {
        unsafe { windows::Win32::Graphics::Gdi::DeleteObject(h_gdi_obj_compatible) };
    }
);

impl_with_acq_and_free_fn!(
    "HGLOBAL_GlobalFree",
    windows::Win32::Foundation::HGLOBAL,
    with_acq_and_global_free,
    with_mut_acq_and_global_free,
    |h_global_compatible| {
        #[cfg(feature = "windows_v0_48")]
        let _ = unsafe { windows::Win32::System::Memory::GlobalFree(h_global_compatible) };

        #[cfg(feature = "windows_v0_52")]
        let _ = unsafe { windows::Win32::Foundation::GlobalFree(h_global_compatible) };
    }
);

impl_with_acq_and_free_fn!(
    "HICON_DestroyIcon",
    windows::Win32::UI::WindowsAndMessaging::HICON,
    with_acq_and_destroy_icon,
    with_mut_acq_and_destroy_icon,
    |h_icon_compatible| {
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyIcon(h_icon_compatible) };
    }
);

impl_with_acq_and_free_fn!(
    "HLOCAL_LocalFree",
    windows::Win32::Foundation::HLOCAL,
    with_acq_and_local_free,
    with_mut_acq_and_local_free,
    |h_local_compatible| {
        #[cfg(feature = "windows_v0_48")]
        let _ = unsafe { windows::Win32::System::Memory::LocalFree(h_local_compatible) };

        #[cfg(feature = "windows_v0_52")]
        let _ = unsafe { windows::Win32::Foundation::LocalFree(h_local_compatible) };
    }
);

impl_with_acq_and_free_fn!(
    "HMENU_DestroyMenu",
    windows::Win32::UI::WindowsAndMessaging::HMENU,
    with_acq_and_destroy_menu,
    with_mut_acq_and_destroy_menu,
    |h_menu_compatible| {
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyMenu(h_menu_compatible) };
    }
);

impl_with_acq_and_free_fn!(
    "HMODULE_FreeLibrary",
    windows::Win32::Foundation::HMODULE,
    with_acq_and_free_library,
    with_mut_acq_and_free_library,
    |h_module_compatible| {
        #[cfg(feature = "windows_v0_48")]
        let _ = unsafe { windows::Win32::System::LibraryLoader::FreeLibrary(h_module_compatible) };

        #[cfg(feature = "windows_v0_52")]
        let _ = unsafe { windows::Win32::Foundation::FreeLibrary(h_module_compatible) };
    }
);

#[cfg(feature = "HANDLE_CloseHandle")]
impl<R> ResGuard<R, fn(R)>
where
    R: windows::core::CanInto<windows::Win32::Foundation::HANDLE>
        + windows::core::TypeKind<TypeKind = windows::core::CopyType>
        + Clone,
{
    const FREE_FN: fn(R) = |handle_compatible| {
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(handle_compatible) };
    };

    pub fn two_with_mut_acq_and_close_handle<A, T, E>(
        acquire_both: A,
    ) -> Result<(ResGuard<R, fn(R)>, ResGuard<R, fn(R)>), E>
    where
        A: FnOnce(&mut R, &mut R) -> Result<T, E>,
        R: Default,
    {
        //! For a function like `CreatePipe()` that returns two resources at once.
        //!
        //! Activate feature `windows_<version>_HANDLE_CloseHandle`.

        Self::two_with_mut_acquisition(acquire_both, Self::FREE_FN, Self::FREE_FN)
    }
}

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
        self.free_fn.take().unwrap()(self.resource.take().unwrap());
    }
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use crate::{
        error::ResultExt,
        windows::{
            self,
            core::PCWSTR,
            Win32::{
                Foundation::{CloseHandle, COLORREF},
                Graphics::Gdi::{CreateSolidBrush, GetObjectW, LOGBRUSH},
                Storage::FileSystem::{ReadFile, WriteFile},
                System::{
                    Pipes::CreatePipe,
                    Threading::{CreateEventW, SetEvent},
                },
            },
        },
    };
    use std::{mem, ptr};

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
    fn with_acq_and_close_handle() {
        let event_handle = ResGuard::with_acq_and_close_handle(|| unsafe {
            CreateEventW(None, true, false, PCWSTR::null())
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
        const BGR: u32 = 0x123456;

        let h_brush = ResGuard::with_acq_and_delete_object(|| unsafe {
            Ok::<_, windows::core::Error>(CreateSolidBrush(COLORREF(BGR)))
        })?;

        let mut log_brush = LOGBRUSH::default();
        Result::from_nonzero_or_win32(unsafe {
            GetObjectW(
                *h_brush,
                mem::size_of::<LOGBRUSH>() as _,
                Some(ptr::addr_of_mut!(log_brush).cast()),
            )
        })?;

        assert_eq!(log_brush.lbColor.0, BGR);

        Ok(())
    }
}

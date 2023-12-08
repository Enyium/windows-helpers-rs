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

    //TODO: Add these and potentially more (with feature gates, analogous to `windows` crate):
    //      with_[mut_]acq_and_close_handle()
    //      with_[mut_]acq_and_free_library()
    //      with_[mut_]acq_and_global_free()
    //      with_[mut_]acq_and_local_free()
    //      with_[mut_]acq_and_heap_free()
    //      with_[mut_]acq_and_delete_object()
    //      with_[mut_]acq_and_release_dc()
    //      with_[mut_]acq_and_destroy_icon()
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
    fn with_acquisition() {
        let event_handle = ResGuard::with_acquisition(
            || unsafe { CreateEventW(None, true, false, PCWSTR::null()) },
            |handle| {
                let _ = unsafe { CloseHandle(handle) };
            },
        )
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

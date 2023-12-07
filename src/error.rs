use num_traits::Zero;
use windows::core::HRESULT;

pub trait ResultExt<T> {
    /// Passes a non-zero `T` through to an `Ok` value, or, in case of it being zero, returns `Err` with [`windows::core::Error::from_win32()`].
    fn from_nonzero_or_win32(t: T) -> windows::core::Result<T>
    where
        T: Zero;
}

impl<T> ResultExt<T> for windows::core::Result<T> {
    fn from_nonzero_or_win32(t: T) -> windows::core::Result<T>
    where
        T: Zero,
    {
        if t.is_zero() {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(t)
        }
    }
}

pub trait HResultExt {
    /// Like `ok()`, but with success `HRESULT`s forwarded instead of giving `()`. Useful when working with functions that can return multiple success return values, like `AssocQueryStringW()`.
    fn ok_with_hresult(self) -> windows::core::Result<HRESULT>;
}

impl HResultExt for HRESULT {
    fn ok_with_hresult(self) -> windows::core::Result<HRESULT> {
        if self.is_ok() {
            Ok(self)
        } else {
            Err(self.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, E_FAIL, E_UNEXPECTED, S_FALSE, S_OK},
        Globalization::{
            GetLocaleInfoEx, LOCALE_ICURRDIGITS, LOCALE_NAME_INVARIANT, LOCALE_RETURN_NUMBER,
        },
    };

    use crate::error::{HResultExt, ResultExt};

    #[test]
    fn result_ext_from_nonzero_or_win32() {
        let mut two_wide_chars = [u16::MAX, u16::MAX];
        assert_eq!(
            windows::core::Result::from_nonzero_or_win32(unsafe {
                GetLocaleInfoEx(
                    LOCALE_NAME_INVARIANT,
                    LOCALE_ICURRDIGITS | LOCALE_RETURN_NUMBER,
                    Some(&mut two_wide_chars),
                )
            }),
            Ok(2)
        );
        let value = unsafe { *two_wide_chars.as_ptr().cast::<u32>() };
        assert!(value <= 2);

        assert_eq!(
            windows::core::Result::from_nonzero_or_win32(unsafe {
                GetLocaleInfoEx(
                    LOCALE_NAME_INVARIANT,
                    LOCALE_ICURRDIGITS | LOCALE_RETURN_NUMBER,
                    Some(&mut [0]),
                )
            }),
            Err(ERROR_INSUFFICIENT_BUFFER.into())
        );
    }

    #[test]
    fn hresult_ext_ok_with_hresult() {
        assert_eq!(S_OK.ok_with_hresult(), Ok(S_OK));
        assert_eq!(S_FALSE.ok_with_hresult(), Ok(S_FALSE));
        assert_eq!(E_FAIL.ok_with_hresult(), Err(E_FAIL.into()));
        assert_eq!(E_UNEXPECTED.ok_with_hresult(), Err(E_UNEXPECTED.into()));
    }
}

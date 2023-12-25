use crate::windows;
use windows::{
    core::HRESULT,
    Win32::Foundation::{E_UNEXPECTED, WIN32_ERROR},
};

pub fn dual_call<F, T>(
    first_call_expectation: FirstCallExpectation<T>,
    mut call: F,
) -> windows::core::Result<T>
where
    F: FnMut(bool) -> windows::core::Result<T>,
    T: PartialEq,
{
    //! For functions that are to be called with a preparation step, normally to determine the required buffer size.
    //!
    //! You may find that this is easier to verify for correctness than a `loop` approach - so, less straining on the mind and less time-consuming. It's also more versatile than a `for` approach.
    //!
    //! The closure parameter will be `true` for the first call. It can be called something like `getting_buffer_size`.
    //!
    //! If the expectation after the first call isn't met and it returned an `Err`, the function ends with that `Err`. If the first call returned `Ok`, however, and this didn't harmonize with the expectation, `Err` including `HRESULT` `E_UNEXPECTED` is returned.

    match first_call_expectation {
        FirstCallExpectation::Ok => {
            call(true)?;
            call(false)
        }
        FirstCallExpectation::OkValue(expected_value) => {
            if call(true)? == expected_value {
                call(false)
            } else {
                Err(E_UNEXPECTED.into())
            }
        }
        other_expectation => {
            let expected_h_result = match other_expectation {
                FirstCallExpectation::Win32Error(win_32_error) => win_32_error.to_hresult(),
                FirstCallExpectation::HResultError(h_result) => h_result,
                _ => unreachable!(),
            };

            match call(true) {
                Err(error) => {
                    if error.code() == expected_h_result {
                        call(false)
                    } else {
                        Err(error)
                    }
                }
                Ok(_) => Err(E_UNEXPECTED.into()),
            }
        }
    }
}

/// Defining the return value of the first call of [`dual_call()`] that is the precondition to continue with the second call.
#[non_exhaustive]
pub enum FirstCallExpectation<T> {
    /// Useful with a function like `GetKeyboardLayoutList()`.
    Ok,
    /// Useful with a function like `AssocQueryStringW()`.
    OkValue(T),
    /// The most useful. Requires `ERROR_INSUFFICIENT_BUFFER` most often, if not documented.
    Win32Error(WIN32_ERROR),
    /// Useful with a function like `AssocQueryStringW()` (in `ASSOCF_NOTRUNCATE` mode).
    HResultError(HRESULT),
}

#[cfg(all(test, feature = "windows_latest_compatible_all"))]
mod tests {
    use super::{dual_call, FirstCallExpectation};
    use crate::{
        core::{CheckNumberError, HResultExt},
        windows, Null, ResGuard,
    };
    use regex::Regex;
    use windows::{
        core::{w, PCWSTR, PWSTR},
        Win32::{
            Foundation::{
                ERROR_BUFFER_OVERFLOW, ERROR_INSUFFICIENT_BUFFER, ERROR_MORE_DATA, E_FAIL,
                E_POINTER, S_FALSE, S_OK, WIN32_ERROR,
            },
            NetworkManagement::IpHelper::{
                GetAdaptersAddresses, GET_ADAPTERS_ADDRESSES_FLAGS, IP_ADAPTER_ADDRESSES_LH,
            },
            Networking::WinSock::AF_UNSPEC,
            Security::{
                Authorization::ConvertSidToStringSidW, GetTokenInformation, TokenUser,
                SID_AND_ATTRIBUTES, TOKEN_QUERY,
            },
            System::{
                SystemInformation::{ComputerNameNetBIOS, GetComputerNameExW},
                Threading::{GetCurrentProcess, OpenProcessToken},
            },
            UI::{
                Input::KeyboardAndMouse::GetKeyboardLayoutList,
                Shell::{AssocQueryStringW, ASSOCF_NONE, ASSOCF_NOTRUNCATE, ASSOCSTR_EXECUTABLE},
                TextServices::HKL,
            },
        },
    };

    #[test]
    fn expect_ok() -> windows::core::Result<()> {
        let mut ids = Vec::<HKL>::new();
        let mut num_ids = 0; // Will equal number of input locales in Windows UI.

        dual_call(FirstCallExpectation::Ok, |getting_buffer_size| {
            num_ids = unsafe {
                GetKeyboardLayoutList((!getting_buffer_size).then(|| {
                    ids.resize(num_ids as _, HKL::default());
                    ids.as_mut_slice()
                }))
            };

            num_ids.nonzero_or_win32_err()
        })?;

        assert!(num_ids >= 1 && num_ids <= 20 && ids.iter().all(|hkl| !hkl.is_invalid()));

        Ok(())
    }

    #[test]
    fn expect_win32_error_more_data() -> windows::core::Result<()> {
        let mut buffer = Vec::new();
        let mut len = 0;

        dual_call(
            FirstCallExpectation::Win32Error(ERROR_MORE_DATA),
            |getting_buffer_size| unsafe {
                GetComputerNameExW(
                    ComputerNameNetBIOS,
                    if getting_buffer_size {
                        PWSTR::NULL
                    } else {
                        buffer.resize(len as _, 0);
                        PWSTR(buffer.as_mut_ptr())
                    },
                    &mut len,
                )
            },
        )?;

        let computer_name = String::from_utf16(&buffer[..len as _])?;
        assert!(
            Regex::new(r"^[\w!@#$%^()\-'{}\.~]{1,15}$") // https://stackoverflow.com/a/24095455
                .unwrap()
                .is_match(&computer_name)
        );

        Ok(())
    }

    #[test]
    fn expect_win32_error_insufficient_buffer() -> windows::core::Result<()> {
        let process_token_handle = ResGuard::with_mut_acq_and_close_handle(|handle| unsafe {
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, handle)
        })?;

        let mut sid_and_attrs_buffer = Vec::<u8>::new();
        let mut sid_and_attrs_buffer_size = 0;

        dual_call(
            FirstCallExpectation::Win32Error(ERROR_INSUFFICIENT_BUFFER),
            |getting_buffer_size| unsafe {
                GetTokenInformation(
                    *process_token_handle,
                    TokenUser,
                    (!getting_buffer_size).then(|| {
                        sid_and_attrs_buffer.resize(sid_and_attrs_buffer_size as _, 0);
                        sid_and_attrs_buffer.as_mut_ptr().cast()
                    }),
                    sid_and_attrs_buffer_size,
                    &mut sid_and_attrs_buffer_size,
                )
            },
        )?;

        let string_sid = unsafe {
            ResGuard::<PWSTR>::with_mut_acq_and_local_free(|pwstr| {
                ConvertSidToStringSidW(
                    (&*sid_and_attrs_buffer.as_ptr().cast::<SID_AND_ATTRIBUTES>()).Sid,
                    pwstr,
                )
            })?
            .to_string()?
        };

        assert!(Regex::new(r"^S-1-5(?:-\d+)+$")
            .unwrap()
            .is_match(&string_sid));

        Ok(())
    }

    #[test]
    fn expect_win32_error_buffer_overflow_from_return() -> windows::core::Result<()> {
        let mut byte_buffer = Vec::<u8>::new();
        let mut buffer_size = 0;

        dual_call(
            FirstCallExpectation::Win32Error(ERROR_BUFFER_OVERFLOW),
            |getting_buffer_size| {
                WIN32_ERROR(unsafe {
                    GetAdaptersAddresses(
                        AF_UNSPEC.0 as _,
                        GET_ADAPTERS_ADDRESSES_FLAGS(0),
                        None,
                        (!getting_buffer_size).then(|| {
                            byte_buffer.resize(buffer_size as _, 0);
                            byte_buffer.as_mut_ptr().cast()
                        }),
                        &mut buffer_size,
                    )
                })
                .to_hresult()
                .ok()
            },
        )?;

        let mut adapter_names = Vec::new();
        let mut ip_adapter_addresses =
            unsafe { &*byte_buffer.as_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>() };

        loop {
            let adapter_name = unsafe { ip_adapter_addresses.FriendlyName.to_string()? };
            if !adapter_names.contains(&adapter_name) {
                adapter_names.push(adapter_name);
            }

            if ip_adapter_addresses.Next.is_null() {
                break;
            }
            ip_adapter_addresses = unsafe { &*ip_adapter_addresses.Next };
        }

        let validate_regex = Regex::new(r"^[\x20-\x7f\p{Letter}]+$").unwrap();
        assert!(adapter_names
            .iter()
            .all(|name| validate_regex.is_match(&name)));

        Ok(())
    }

    #[test]
    fn expect_ok_value() -> windows::core::Result<()> {
        let mut buffer = Vec::new();
        let mut buffer_size = 0;

        let success_hresult = dual_call(
            FirstCallExpectation::OkValue(S_FALSE),
            |getting_buffer_size| {
                unsafe {
                    AssocQueryStringW(
                        ASSOCF_NONE,
                        ASSOCSTR_EXECUTABLE,
                        w!(".msi"),
                        PCWSTR::NULL,
                        if getting_buffer_size {
                            PWSTR::NULL
                        } else {
                            buffer.resize(buffer_size as _, 0);
                            PWSTR(buffer.as_mut_ptr())
                        },
                        &mut buffer_size,
                    )
                }
                .ok_with_hresult()
            },
        )?;

        if success_hresult == S_OK && buffer_size > 0 {
            let string = String::from_utf16(&buffer[..(buffer_size - 1) as _])?;
            assert!(Regex::new(r"(?i)\\System32\\msiexec.exe$")
                .unwrap()
                .is_match(&string));
        } else {
            return Err(E_FAIL.into());
        }

        Ok(())
    }

    #[test]
    fn expect_hresult_error() -> windows::core::Result<()> {
        let mut buffer = Vec::new();
        let mut buffer_size = 0;

        let success_hresult = dual_call(
            FirstCallExpectation::HResultError(E_POINTER),
            |getting_buffer_size| {
                if !getting_buffer_size {
                    buffer.resize(buffer_size as _, 0);
                }

                unsafe {
                    AssocQueryStringW(
                        ASSOCF_NOTRUNCATE,
                        ASSOCSTR_EXECUTABLE,
                        w!(".msi"),
                        PCWSTR::NULL,
                        PWSTR(buffer.as_mut_ptr()),
                        &mut buffer_size,
                    )
                }
                .ok_with_hresult()
            },
        )?;

        if success_hresult == S_OK && buffer_size > 0 {
            let string = String::from_utf16(&buffer[..(buffer_size - 1) as _])?;
            assert!(Regex::new(r"(?i)\\System32\\msiexec.exe$")
                .unwrap()
                .is_match(&string));
        } else {
            return Err(E_FAIL.into());
        }

        Ok(())
    }
}

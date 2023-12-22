use crate::windows;
use windows::core::HSTRING;

pub trait HStringExt {
    /// Similar to `HSTRING::as_wide()`, but truncates the slice to the specified length, avoiding to cut a UTF-16 surrogate pair in half by reducing the length by one additional wide char, if needed.
    ///
    /// With a target length larger than the string length, the behavior is identical to that of `as_wide()`.
    fn as_safely_truncated_wide(&self, target_len: usize) -> &[u16];

    /// Writes the `HSTRING` into the buffer, followed by a terminating null character.
    fn write_truncated(&self, buf: &mut [u16]);
}

impl HStringExt for HSTRING {
    fn as_safely_truncated_wide(&self, target_len: usize) -> &[u16] {
        let slice = self.as_wide();
        let len = slice.len();

        if target_len >= len {
            // Nothing to do.
            return slice;
        }

        if len
            .checked_sub(1)
            .map(|last_index| is_leading_surrogate(slice[last_index]))
            .unwrap_or(false /*empty string doesn't end in leading surrogate*/)
        {
            // Don't cut surrogate pair in half. Remove whole pair instead.
            &slice[..len - 1]
        } else {
            // Ordinary truncation.
            &slice[..len]
        }
    }

    fn write_truncated(&self, buf: &mut [u16]) {
        let truncated_slice =
            self.as_safely_truncated_wide(self.len().min(buf.len() - 1 /*null-termination*/));

        buf[..truncated_slice.len()].copy_from_slice(truncated_slice);
        buf[truncated_slice.len()] = 0;
    }
}

const fn is_leading_surrogate(wide_char: u16) -> bool {
    wide_char >= 0xd800 && wide_char <= 0xdbff
}

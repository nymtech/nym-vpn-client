use windows::{
    Win32::{
        Foundation::ERROR_SUCCESS,
        System::Diagnostics::Debug::{
            FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageW,
        },
    },
    core::PWSTR,
};

/// Get a descriptive string for a Win32 error code.
pub fn win32_error(code: u32) -> String {
    let descr = {
        if code == ERROR_SUCCESS.0 {
            "The operation completed successfully.".to_string()
        } else {
            let mut buf: [u16; 512] = [0; 512];
            let flags = FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS;
            let len = unsafe {
                FormatMessageW(
                    flags,
                    None,
                    code,
                    0,
                    PWSTR(buf.as_mut_ptr()),
                    buf.len() as u32,
                    None,
                )
            };
            if len == 0 {
                "(unrecognized Win32 error)".to_string()
            } else {
                let msg = String::from_utf16_lossy(&buf[..len as usize]);
                msg.trim().trim_end_matches('.').to_string()
            }
        }
    };
    format!("{descr} (0x{code:08x})")
}

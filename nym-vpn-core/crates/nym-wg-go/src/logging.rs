use std::ffi::{c_char, c_void, CStr};

pub unsafe extern "system" fn wg_logger_callback(
    _log_level: u32,
    msg: *const c_char,
    ctx: *mut c_void,
) {
    if !ctx.is_null() && !msg.is_null() {
        // Reckless cast to box with Fn, which should work for as long as the call site plays by the rules.
        let closure = ctx as *mut Box<dyn Fn(&str)>;

        // We expect utf8.
        let str = CStr::from_ptr(msg).to_string_lossy();

        (*closure)(str.as_ref());
    }
}

pub unsafe fn create_logger_callback<F>(f: F) -> *mut Box<dyn Fn(&str)>
where
    F: Fn(&str) + 'static,
{
    // Double box to have a normal size pointer that we can feed into C.
    let boxed_logger = Box::new(Box::new(f) as Box<dyn Fn(&str)>);
    Box::into_raw(boxed_logger)
}

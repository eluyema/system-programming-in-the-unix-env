use std::ffi::CString;
use crate::exit_err;

pub fn str_to_cstring(s: &str) -> CString {
    let mut v = Vec::<u8>::new();
    if let Err(e) = v.try_reserve_exact(s.len() + 1) {
        exit_err!("str_to_cstring for string = {s}");
    }
    v.extend_from_slice(s.as_bytes());
    unsafe { CString::from_vec_unchecked(v) }
}

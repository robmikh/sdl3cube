use std::str::Utf8Error;

pub fn null_terminated_sdl_str<'a>(start: *const i8) -> Result<Option<&'a str>, Utf8Error> {
    if start != std::ptr::null() {
        unsafe {
            let mut end = start;
            while *end != 0 {
                end = end.add(1);
            }
            let len = end.offset_from(start);
            let str_slice = std::slice::from_raw_parts(start as *const u8, len as usize);
            std::str::from_utf8(str_slice).map(|x| Some(x))
        }
    } else {
        Ok(None)
    }
}

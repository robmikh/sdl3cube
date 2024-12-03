use sdl3_sys::error::SDL_GetError;

pub type SdlResult<T> = std::result::Result<T, SdlError>;

#[derive(Debug)]
pub struct SdlError {
    pub message: String,
}

impl std::error::Error for SdlError {}

impl std::fmt::Display for SdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl SdlError {
    fn get_current() -> Self {
        let message = unsafe {
            let error_msg_start = SDL_GetError();
            let mut error_msg_end = error_msg_start;
            if error_msg_start != std::ptr::null() {
                while *error_msg_end != 0 {
                    error_msg_end = error_msg_end.add(1);
                }
            }
            let len = error_msg_end.offset_from(error_msg_start);
            let error_msg_slice =
                std::slice::from_raw_parts(error_msg_start as *const u8, len as usize);
            match std::str::from_utf8(error_msg_slice) {
                Ok(msg) => format!("SDL Error: {}", msg.to_owned()),
                Err(error) => format!("Utf8Error: {}", error),
            }
        };
        Self { message }
    }
}

pub trait SdlFunctionResult<T> {
    fn ok(self) -> SdlResult<T>;
}

impl SdlFunctionResult<()> for std::primitive::bool {
    fn ok(self) -> SdlResult<()> {
        if self {
            Ok(())
        } else {
            Err(SdlError::get_current())
        }
    }
}

impl<T> SdlFunctionResult<*mut T> for *mut T {
    fn ok(self) -> SdlResult<*mut T> {
        if !self.is_null() {
            Ok(self)
        } else {
            Err(SdlError::get_current())
        }
    }
}

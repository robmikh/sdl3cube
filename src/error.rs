use sdl3_sys::error::SDL_GetError;

use crate::util::null_terminated_sdl_str;

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
            match null_terminated_sdl_str(SDL_GetError()) {
                Ok(msg) => format!("SDL Error: {}", msg.unwrap_or("[None]")),
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

impl<T> SdlFunctionResult<*const T> for *const T {
    fn ok(self) -> SdlResult<*const T> {
        if !self.is_null() {
            Ok(self)
        } else {
            Err(SdlError::get_current())
        }
    }
}

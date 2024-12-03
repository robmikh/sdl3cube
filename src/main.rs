use core::str;

use sdl3_sys::{error::SDL_GetError, init::{SDL_Init, SDL_INIT_VIDEO}};

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;

fn main() -> SdlResult<()> {
    init()?;

    Ok(())
}

type SdlResult<T> = std::result::Result<T, SdlError>;

#[derive(Debug)]
struct SdlError {
    message: String,
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
            let error_msg_slice = std::slice::from_raw_parts(error_msg_start as *const u8, len as usize);
            match str::from_utf8(error_msg_slice) {
                Ok(msg) => format!("SDL Error: {}", msg.to_owned()),
                Err(error) => format!("Utf8Error: {}", error),
            }
        };
        Self {
            message,
        }
    }
}

fn init() -> SdlResult<()> {
    let init_result = unsafe {
        SDL_Init(SDL_INIT_VIDEO)
    };
    if !init_result {
        return Err(SdlError::get_current());
    }
    Ok(())
}
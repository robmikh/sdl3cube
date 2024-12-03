mod error;

use error::{SdlFunctionResult, SdlResult};
use sdl3_sys::init::{SDL_Init, SDL_INIT_VIDEO};

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;

fn main() -> SdlResult<()> {
    init()?;

    

    Ok(())
}

fn init() -> SdlResult<()> {
    unsafe {
        SDL_Init(SDL_INIT_VIDEO).ok()?;
    }
    Ok(())
}
mod error;

use error::{SdlFunctionResult, SdlResult};
use sdl3_sys::{
    events::{SDL_Event, SDL_PollEvent, SDL_EVENT_QUIT},
    init::{SDL_Init, SDL_INIT_VIDEO},
    video::{SDL_CreateWindow, SDL_DestroyWindow},
};

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

fn main() -> SdlResult<()> {
    init()?;

    let window = unsafe {
        SDL_CreateWindow(
            "sdl3cube\0".as_ptr() as *const _,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            0,
        )
        .ok()?
    };

    let mut quit = false;
    let mut event = SDL_Event {
        padding: [0u8; 128],
    };
    while !quit {
        while unsafe { SDL_PollEvent(&mut event) } {
            if unsafe { event.r#type } == SDL_EVENT_QUIT.0 {
                quit = true;
                break;
            }
        }
    }

    unsafe {
        SDL_DestroyWindow(window);
    }

    Ok(())
}

fn init() -> SdlResult<()> {
    unsafe {
        SDL_Init(SDL_INIT_VIDEO).ok()?;
    }
    Ok(())
}

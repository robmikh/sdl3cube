mod error;
mod util;

use error::{SdlFunctionResult, SdlResult};
use sdl3_sys::{
    events::{SDL_Event, SDL_PollEvent, SDL_EVENT_QUIT},
    gpu::{
        SDL_ClaimWindowForGPUDevice, SDL_CreateGPUDevice, SDL_CreateGPUShader,
        SDL_DestroyGPUDevice, SDL_GPUShaderCreateInfo, SDL_GetGPUDeviceDriver,
        SDL_GPU_SHADERFORMAT_SPIRV, SDL_GPU_SHADERSTAGE_FRAGMENT, SDL_GPU_SHADERSTAGE_VERTEX,
    },
    init::{SDL_Init, SDL_Quit, SDL_INIT_VIDEO},
    video::{SDL_CreateWindow, SDL_DestroyWindow},
};
use util::null_terminated_sdl_str;

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

const SHADER_BYTES: &[u8] = include_bytes!("../data/generated/shaders/vertex.spv");

fn main() -> SdlResult<()> {
    // Init SDL
    unsafe {
        SDL_Init(SDL_INIT_VIDEO).ok()?;
    }

    // Create our window
    let window = unsafe {
        SDL_CreateWindow(
            "sdl3cube\0".as_ptr() as *const _,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            0,
        )
        .ok()?
    };

    // Init GPU
    let device =
        unsafe { SDL_CreateGPUDevice(SDL_GPU_SHADERFORMAT_SPIRV, true, std::ptr::null()).ok()? };

    // Log the backend
    let device_backend = unsafe {
        null_terminated_sdl_str(SDL_GetGPUDeviceDriver(device).ok()?)
            .unwrap()
            .unwrap()
    };
    println!("GPU backend: {}", device_backend);

    // Associate our device with out window
    unsafe {
        SDL_ClaimWindowForGPUDevice(device, window).ok()?;
    }

    // Load our shaders
    let vertex_shader = unsafe {
        let desc = SDL_GPUShaderCreateInfo {
            code_size: SHADER_BYTES.len(),
            code: SHADER_BYTES.as_ptr(),
            entrypoint: "vs_main\0".as_ptr() as *const _,
            format: SDL_GPU_SHADERFORMAT_SPIRV,
            stage: SDL_GPU_SHADERSTAGE_VERTEX,
            num_samplers: 0,
            num_storage_textures: 0,
            num_storage_buffers: 0,
            num_uniform_buffers: 0,
            props: 0,
        };
        SDL_CreateGPUShader(device, &desc).ok()?
    };
    let fragment_shader = unsafe {
        let desc = SDL_GPUShaderCreateInfo {
            code_size: SHADER_BYTES.len(),
            code: SHADER_BYTES.as_ptr(),
            entrypoint: "fs_main\0".as_ptr() as *const _,
            format: SDL_GPU_SHADERFORMAT_SPIRV,
            stage: SDL_GPU_SHADERSTAGE_FRAGMENT,
            num_samplers: 0,
            num_storage_textures: 0,
            num_storage_buffers: 0,
            num_uniform_buffers: 0,
            props: 0,
        };
        SDL_CreateGPUShader(device, &desc).ok()?
    };

    // Message pump
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

    // TODO: RAII-like wrapers
    unsafe {
        SDL_DestroyGPUDevice(device);
        SDL_DestroyWindow(window);
        SDL_Quit();
    }

    Ok(())
}

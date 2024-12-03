mod error;
mod util;

use error::{SdlFunctionResult, SdlResult};
use sdl3_sys::{
    events::{SDL_Event, SDL_PollEvent, SDL_EVENT_QUIT},
    gpu::{
        SDL_ClaimWindowForGPUDevice, SDL_CreateGPUDevice, SDL_CreateGPUGraphicsPipeline,
        SDL_CreateGPUShader, SDL_DestroyGPUDevice, SDL_GPUColorTargetBlendState,
        SDL_GPUColorTargetDescription, SDL_GPUDepthStencilState, SDL_GPUGraphicsPipelineCreateInfo,
        SDL_GPUGraphicsPipelineTargetInfo, SDL_GPUMultisampleState, SDL_GPURasterizerState,
        SDL_GPUSampleCount, SDL_GPUShaderCreateInfo, SDL_GPUStencilOpState, SDL_GPUVertexAttribute,
        SDL_GPUVertexBufferDescription, SDL_GPUVertexInputState, SDL_GetGPUDeviceDriver,
        SDL_GPU_BLENDFACTOR_ONE, SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
        SDL_GPU_BLENDFACTOR_SRC_ALPHA, SDL_GPU_BLENDOP_ADD, SDL_GPU_COMPAREOP_INVALID,
        SDL_GPU_CULLMODE_BACK, SDL_GPU_FILLMODE_FILL, SDL_GPU_FRONTFACE_CLOCKWISE,
        SDL_GPU_PRIMITIVETYPE_TRIANGLELIST, SDL_GPU_SHADERFORMAT_SPIRV,
        SDL_GPU_SHADERSTAGE_FRAGMENT, SDL_GPU_SHADERSTAGE_VERTEX, SDL_GPU_STENCILOP_INVALID,
        SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM, SDL_GPU_TEXTUREFORMAT_INVALID,
        SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4, SDL_GPU_VERTEXINPUTRATE_VERTEX,
    },
    init::{SDL_Init, SDL_Quit, SDL_INIT_VIDEO},
    video::{SDL_CreateWindow, SDL_DestroyWindow},
};
use util::null_terminated_sdl_str;

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

const VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../data/generated/shaders/vertex.spv");
const FRAGMENT_SHADER_BYTES: &[u8] = include_bytes!("../data/generated/shaders/fragment.spv");

#[repr(C)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}

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
            code_size: VERTEX_SHADER_BYTES.len(),
            code: VERTEX_SHADER_BYTES.as_ptr(),
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
            code_size: FRAGMENT_SHADER_BYTES.len(),
            code: FRAGMENT_SHADER_BYTES.as_ptr(),
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

    // Create our pipeline
    let pipeline = unsafe {
        let vertex_buffer_descriptions = [SDL_GPUVertexBufferDescription {
            slot: 0,
            pitch: std::mem::size_of::<Vertex>() as u32,
            input_rate: SDL_GPU_VERTEXINPUTRATE_VERTEX,
            instance_step_rate: 0,
        }];
        let vertex_attributes = [
            SDL_GPUVertexAttribute {
                location: 0,
                buffer_slot: 0,
                format: SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4,
                offset: 0,
            },
            SDL_GPUVertexAttribute {
                location: 1,
                buffer_slot: 0,
                format: SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4,
                offset: 4 * 4,
            },
        ];

        let vertex_input_state = SDL_GPUVertexInputState {
            vertex_buffer_descriptions: vertex_buffer_descriptions.as_ptr(),
            num_vertex_buffers: vertex_buffer_descriptions.len() as u32,
            vertex_attributes: vertex_attributes.as_ptr(),
            num_vertex_attributes: vertex_attributes.len() as u32,
        };

        let color_targets = [SDL_GPUColorTargetDescription {
            format: SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM,
            blend_state: SDL_GPUColorTargetBlendState {
                src_color_blendfactor: SDL_GPU_BLENDFACTOR_SRC_ALPHA,
                dst_color_blendfactor: SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
                color_blend_op: SDL_GPU_BLENDOP_ADD,
                src_alpha_blendfactor: SDL_GPU_BLENDFACTOR_ONE,
                dst_alpha_blendfactor: SDL_GPU_BLENDFACTOR_ONE,
                alpha_blend_op: SDL_GPU_BLENDOP_ADD,
                color_write_mask: 0,
                enable_blend: true,
                enable_color_write_mask: false,
                padding1: 0,
                padding2: 0,
            },
        }];

        let desc = SDL_GPUGraphicsPipelineCreateInfo {
            vertex_shader,
            fragment_shader,
            vertex_input_state,
            primitive_type: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            rasterizer_state: SDL_GPURasterizerState {
                fill_mode: SDL_GPU_FILLMODE_FILL,
                cull_mode: SDL_GPU_CULLMODE_BACK,
                front_face: SDL_GPU_FRONTFACE_CLOCKWISE,
                depth_bias_constant_factor: 0.0,
                depth_bias_clamp: 0.0,
                depth_bias_slope_factor: 0.0,
                enable_depth_bias: false,
                enable_depth_clip: false,
                padding1: 0,
                padding2: 0,
            },
            multisample_state: SDL_GPUMultisampleState {
                sample_count: SDL_GPUSampleCount::_1,
                sample_mask: 0,
                enable_mask: false,
                padding1: 0,
                padding2: 0,
                padding3: 0,
            },
            depth_stencil_state: SDL_GPUDepthStencilState {
                compare_op: SDL_GPU_COMPAREOP_INVALID,
                back_stencil_state: SDL_GPUStencilOpState {
                    fail_op: SDL_GPU_STENCILOP_INVALID,
                    pass_op: SDL_GPU_STENCILOP_INVALID,
                    depth_fail_op: SDL_GPU_STENCILOP_INVALID,
                    compare_op: SDL_GPU_COMPAREOP_INVALID,
                },
                front_stencil_state: SDL_GPUStencilOpState {
                    fail_op: SDL_GPU_STENCILOP_INVALID,
                    pass_op: SDL_GPU_STENCILOP_INVALID,
                    depth_fail_op: SDL_GPU_STENCILOP_INVALID,
                    compare_op: SDL_GPU_COMPAREOP_INVALID,
                },
                compare_mask: 0,
                write_mask: 0,
                enable_depth_test: false,
                enable_depth_write: false,
                enable_stencil_test: false,
                padding1: 0,
                padding2: 0,
                padding3: 0,
            },
            target_info: SDL_GPUGraphicsPipelineTargetInfo {
                color_target_descriptions: color_targets.as_ptr(),
                num_color_targets: color_targets.len() as u32,
                depth_stencil_format: SDL_GPU_TEXTUREFORMAT_INVALID,
                has_depth_stencil_target: false,
                padding1: 0,
                padding2: 0,
                padding3: 0,
            },
            props: 0,
        };
        SDL_CreateGPUGraphicsPipeline(device, &desc).ok()?
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

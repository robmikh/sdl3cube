mod error;
mod util;

use error::{SdlFunctionResult, SdlResult};
use sdl3_sys::{
    events::{SDL_Event, SDL_PollEvent, SDL_EVENT_QUIT},
    gpu::{
        SDL_AcquireGPUCommandBuffer, SDL_BeginGPUCopyPass, SDL_ClaimWindowForGPUDevice,
        SDL_CreateGPUBuffer, SDL_CreateGPUDevice, SDL_CreateGPUGraphicsPipeline,
        SDL_CreateGPUShader, SDL_CreateGPUTransferBuffer, SDL_DestroyGPUDevice, SDL_EndGPUCopyPass,
        SDL_GPUBufferCreateInfo, SDL_GPUBufferRegion, SDL_GPUColorTargetBlendState,
        SDL_GPUColorTargetDescription, SDL_GPUDepthStencilState, SDL_GPUGraphicsPipelineCreateInfo,
        SDL_GPUGraphicsPipelineTargetInfo, SDL_GPUMultisampleState, SDL_GPURasterizerState,
        SDL_GPUSampleCount, SDL_GPUShaderCreateInfo, SDL_GPUStencilOpState,
        SDL_GPUTransferBufferCreateInfo, SDL_GPUTransferBufferLocation, SDL_GPUVertexAttribute,
        SDL_GPUVertexBufferDescription, SDL_GPUVertexInputState, SDL_GetGPUDeviceDriver,
        SDL_MapGPUTransferBuffer, SDL_ReleaseGPUBuffer, SDL_ReleaseGPUFence,
        SDL_ReleaseGPUGraphicsPipeline, SDL_ReleaseGPUShader, SDL_ReleaseGPUTransferBuffer,
        SDL_ReleaseWindowFromGPUDevice, SDL_SubmitGPUCommandBufferAndAcquireFence,
        SDL_UnmapGPUTransferBuffer, SDL_UploadToGPUBuffer, SDL_WaitForGPUFences,
        SDL_GPU_BLENDFACTOR_ONE, SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
        SDL_GPU_BLENDFACTOR_SRC_ALPHA, SDL_GPU_BLENDOP_ADD, SDL_GPU_BUFFERUSAGE_INDEX,
        SDL_GPU_BUFFERUSAGE_VERTEX, SDL_GPU_COMPAREOP_INVALID, SDL_GPU_CULLMODE_BACK,
        SDL_GPU_FILLMODE_FILL, SDL_GPU_FRONTFACE_CLOCKWISE, SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
        SDL_GPU_SHADERFORMAT_SPIRV, SDL_GPU_SHADERSTAGE_FRAGMENT, SDL_GPU_SHADERSTAGE_VERTEX,
        SDL_GPU_STENCILOP_INVALID, SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM,
        SDL_GPU_TEXTUREFORMAT_INVALID, SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
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

impl Vertex {
    fn new(pos: [f32; 4], color: [f32; 4]) -> Self {
        Self { pos, color }
    }
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

    // Create our vertex and index data
    let vertex_data = [
        Vertex::new([0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 1.0]),
        Vertex::new([0.0, 1.0, 0.0, 0.0], [0.0, 1.0, 0.0, 1.0]),
        Vertex::new([1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]),
    ];
    let index_data = [0, 1, 2];

    // Create our vertex and index buffers
    let (vertex_buffer, index_buffer) = unsafe {
        let command_buffer = SDL_AcquireGPUCommandBuffer(device).ok()?;
        let copy_pass = SDL_BeginGPUCopyPass(command_buffer).ok()?;

        let vertex_buffer_size = (vertex_data.len() * std::mem::size_of::<Vertex>()) as u32;
        let index_buffer_size = (index_data.len() * std::mem::size_of::<i32>()) as u32;

        // Create the buffers
        let vertex_buffer = {
            let desc = SDL_GPUBufferCreateInfo {
                usage: SDL_GPU_BUFFERUSAGE_VERTEX,
                size: vertex_buffer_size,
                props: 0,
            };
            SDL_CreateGPUBuffer(device, &desc).ok()?
        };
        let index_buffer = {
            let desc = SDL_GPUBufferCreateInfo {
                usage: SDL_GPU_BUFFERUSAGE_INDEX,
                size: index_buffer_size,
                props: 0,
            };
            SDL_CreateGPUBuffer(device, &desc).ok()?
        };

        let transfer_buffer = {
            let desc = SDL_GPUTransferBufferCreateInfo {
                usage: SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
                size: vertex_buffer_size + index_buffer_size,
                props: 0,
            };
            SDL_CreateGPUTransferBuffer(device, &desc).ok()?
        };

        // Populate our transfer buffer
        {
            let dest_ptr = SDL_MapGPUTransferBuffer(device, transfer_buffer, false).ok()?;
            let dest_slice = std::slice::from_raw_parts_mut(
                dest_ptr as *mut u8,
                (vertex_buffer_size + index_buffer_size) as usize,
            );

            let vertex_bytes = std::slice::from_raw_parts(
                vertex_data.as_ptr() as *const u8,
                vertex_buffer_size as usize,
            );
            let index_bytes = std::slice::from_raw_parts(
                index_data.as_ptr() as *const u8,
                index_buffer_size as usize,
            );

            dest_slice[0..vertex_bytes.len()].copy_from_slice(vertex_bytes);
            dest_slice[vertex_bytes.len()..].copy_from_slice(index_bytes);

            SDL_UnmapGPUTransferBuffer(device, transfer_buffer);
        }

        // Copy to our vertex and index buffers
        {
            let source = SDL_GPUTransferBufferLocation {
                transfer_buffer: transfer_buffer,
                offset: 0,
            };
            let dest = SDL_GPUBufferRegion {
                buffer: vertex_buffer,
                offset: 0,
                size: vertex_buffer_size,
            };
            SDL_UploadToGPUBuffer(copy_pass, &source, &dest, false);
        }

        {
            let source = SDL_GPUTransferBufferLocation {
                transfer_buffer: transfer_buffer,
                offset: vertex_buffer_size,
            };
            let dest = SDL_GPUBufferRegion {
                buffer: index_buffer,
                offset: 0,
                size: index_buffer_size,
            };
            SDL_UploadToGPUBuffer(copy_pass, &source, &dest, false);
        }

        // Execute and wait for the copies
        SDL_EndGPUCopyPass(copy_pass);
        let fence = SDL_SubmitGPUCommandBufferAndAcquireFence(command_buffer).ok()?;

        SDL_WaitForGPUFences(device, true, [fence].as_ptr(), 1).ok()?;
        SDL_ReleaseGPUFence(device, fence);
        SDL_ReleaseGPUTransferBuffer(device, transfer_buffer);

        (vertex_buffer, index_buffer)
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
        SDL_ReleaseGPUShader(device, vertex_shader);
        SDL_ReleaseGPUShader(device, fragment_shader);
        SDL_ReleaseGPUBuffer(device, vertex_buffer);
        SDL_ReleaseGPUBuffer(device, index_buffer);
        SDL_ReleaseGPUGraphicsPipeline(device, pipeline);
        SDL_ReleaseWindowFromGPUDevice(device, window);
        SDL_DestroyGPUDevice(device);
        SDL_DestroyWindow(window);
        SDL_Quit();
    }

    Ok(())
}

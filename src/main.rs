mod cube;
mod error;
mod sdl;
mod util;

use std::time::Instant;

use cube::create_cube;
use error::{SdlFunctionResult, SdlResult};
use glam::{Mat4, Vec3};
use sdl::{
    SdlGpuBuffer, SdlGpuDevice, SdlGpuGraphicsPipeline, SdlGpuShader, SdlGpuTransferBuffer,
    SdlWindow,
};
use sdl3_sys::{
    events::{SDL_Event, SDL_EventType, SDL_PollEvent, SDL_EVENT_KEY_UP, SDL_EVENT_QUIT},
    gpu::{
        SDL_AcquireGPUCommandBuffer, SDL_AcquireGPUSwapchainTexture, SDL_BeginGPUCopyPass,
        SDL_BeginGPURenderPass, SDL_BindGPUGraphicsPipeline, SDL_BindGPUIndexBuffer,
        SDL_BindGPUVertexBuffers, SDL_ClaimWindowForGPUDevice, SDL_CreateGPUBuffer,
        SDL_CreateGPUDevice, SDL_CreateGPUGraphicsPipeline, SDL_CreateGPUShader,
        SDL_CreateGPUTransferBuffer, SDL_DrawGPUIndexedPrimitives, SDL_EndGPUCopyPass,
        SDL_EndGPURenderPass, SDL_GPUBufferBinding, SDL_GPUBufferCreateInfo, SDL_GPUBufferRegion,
        SDL_GPUColorTargetBlendState, SDL_GPUColorTargetDescription, SDL_GPUColorTargetInfo,
        SDL_GPUDepthStencilState, SDL_GPUGraphicsPipelineCreateInfo,
        SDL_GPUGraphicsPipelineTargetInfo, SDL_GPUMultisampleState, SDL_GPURasterizerState,
        SDL_GPUSampleCount, SDL_GPUShaderCreateInfo, SDL_GPUStencilOpState,
        SDL_GPUTransferBufferCreateInfo, SDL_GPUTransferBufferLocation, SDL_GPUVertexAttribute,
        SDL_GPUVertexBufferDescription, SDL_GPUVertexInputState, SDL_GPUViewport,
        SDL_GetGPUDeviceDriver, SDL_MapGPUTransferBuffer, SDL_PushGPUVertexUniformData,
        SDL_ReleaseGPUFence, SDL_ReleaseWindowFromGPUDevice, SDL_SetGPUViewport,
        SDL_SubmitGPUCommandBufferAndAcquireFence, SDL_UnmapGPUTransferBuffer,
        SDL_UploadToGPUBuffer, SDL_WaitForGPUFences, SDL_GPU_BLENDFACTOR_ONE,
        SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA, SDL_GPU_BLENDFACTOR_SRC_ALPHA,
        SDL_GPU_BLENDOP_ADD, SDL_GPU_BUFFERUSAGE_INDEX, SDL_GPU_BUFFERUSAGE_VERTEX,
        SDL_GPU_COMPAREOP_INVALID, SDL_GPU_CULLMODE_BACK, SDL_GPU_FILLMODE_FILL,
        SDL_GPU_FRONTFACE_COUNTER_CLOCKWISE, SDL_GPU_INDEXELEMENTSIZE_32BIT, SDL_GPU_LOADOP_CLEAR,
        SDL_GPU_PRIMITIVETYPE_TRIANGLELIST, SDL_GPU_SHADERSTAGE_FRAGMENT,
        SDL_GPU_SHADERSTAGE_VERTEX, SDL_GPU_STENCILOP_INVALID, SDL_GPU_STOREOP_STORE,
        SDL_GPU_TEXTUREFORMAT_B8G8R8A8_UNORM, SDL_GPU_TEXTUREFORMAT_INVALID,
        SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4,
        SDL_GPU_VERTEXINPUTRATE_VERTEX,
    },
    init::{SDL_Init, SDL_Quit, SDL_INIT_VIDEO},
    keycode::{SDLK_A, SDLK_D, SDLK_E, SDLK_Q, SDLK_S, SDLK_W},
    pixels::SDL_FColor,
    video::{SDL_CreateWindow, SDL_WINDOW_RESIZABLE},
};
use util::null_terminated_sdl_str;

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

// Default to Vulkan
#[cfg(all(not(feature = "vulkan"), not(feature = "dx12"), not(feature = "metal")))]
mod shaders {
    pub const VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../data/generated/shaders/vertex.spv");
    pub const FRAGMENT_SHADER_BYTES: &[u8] =
        include_bytes!("../data/generated/shaders/fragment.spv");
    pub const SHADER_TYPE: sdl3_sys::gpu::SDL_GPUShaderFormat =
        sdl3_sys::gpu::SDL_GPU_SHADERFORMAT_SPIRV;
}

#[cfg(feature = "dx12")]
mod shaders {
    pub const VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../data/generated/shaders/vertex.dxil");
    pub const FRAGMENT_SHADER_BYTES: &[u8] =
        include_bytes!("../data/generated/shaders/fragment.dxil");
    pub const SHADER_TYPE: sdl3_sys::gpu::SDL_GPUShaderFormat =
        sdl3_sys::gpu::SDL_GPU_SHADERFORMAT_DXIL;
}

#[cfg(feature = "vulkan")]
mod shaders {
    pub const VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../data/generated/shaders/vertex.spv");
    pub const FRAGMENT_SHADER_BYTES: &[u8] =
        include_bytes!("../data/generated/shaders/fragment.spv");
    pub const SHADER_TYPE: sdl3_sys::gpu::SDL_GPUShaderFormat =
        sdl3_sys::gpu::SDL_GPU_SHADERFORMAT_SPIRV;
}

#[cfg(feature = "metal")]
mod shaders {
    pub const VERTEX_SHADER_BYTES: &[u8] = include_bytes!("../data/generated/shaders/vertex.msl");
    pub const FRAGMENT_SHADER_BYTES: &[u8] =
        include_bytes!("../data/generated/shaders/fragment.msl");
    pub const SHADER_TYPE: sdl3_sys::gpu::SDL_GPUShaderFormat =
        sdl3_sys::gpu::SDL_GPU_SHADERFORMAT_MSL;
}

#[repr(C)]
pub struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}

impl Vertex {
    pub fn new(pos: [f32; 4], color: [f32; 4]) -> Self {
        Self { pos, color }
    }
}

fn run() -> SdlResult<()> {
    // Create our window
    let window: SdlWindow = unsafe {
        SDL_CreateWindow(
            "sdl3cube\0".as_ptr() as *const _,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            SDL_WINDOW_RESIZABLE,
        )
        .ok()?
        .into()
    };

    // Init GPU
    let device: SdlGpuDevice = unsafe {
        SDL_CreateGPUDevice(shaders::SHADER_TYPE, true, std::ptr::null())
            .ok()?
            .into()
    };

    // Log the backend
    let device_backend = unsafe {
        null_terminated_sdl_str(SDL_GetGPUDeviceDriver(device.0).ok()?)
            .unwrap()
            .unwrap()
    };
    println!("GPU backend: {}", device_backend);

    // Associate our device with out window
    unsafe {
        SDL_ClaimWindowForGPUDevice(device.0, window.0).ok()?;
    }

    // Load our shaders
    let vertex_shader = unsafe {
        let desc = SDL_GPUShaderCreateInfo {
            code_size: shaders::VERTEX_SHADER_BYTES.len(),
            code: shaders::VERTEX_SHADER_BYTES.as_ptr(),
            entrypoint: "vs_main\0".as_ptr() as *const _,
            format: shaders::SHADER_TYPE,
            stage: SDL_GPU_SHADERSTAGE_VERTEX,
            num_samplers: 0,
            num_storage_textures: 0,
            num_storage_buffers: 0,
            num_uniform_buffers: 2,
            props: 0,
        };
        let shader = SDL_CreateGPUShader(device.0, &desc).ok()?;
        SdlGpuShader::new(shader, device.0)
    };
    let fragment_shader = unsafe {
        let desc = SDL_GPUShaderCreateInfo {
            code_size: shaders::FRAGMENT_SHADER_BYTES.len(),
            code: shaders::FRAGMENT_SHADER_BYTES.as_ptr(),
            entrypoint: "fs_main\0".as_ptr() as *const _,
            format: shaders::SHADER_TYPE,
            stage: SDL_GPU_SHADERSTAGE_FRAGMENT,
            num_samplers: 0,
            num_storage_textures: 0,
            num_storage_buffers: 0,
            num_uniform_buffers: 0,
            props: 0,
        };
        let shader = SDL_CreateGPUShader(device.0, &desc).ok()?;
        SdlGpuShader::new(shader, device.0)
    };

    // Create our vertex and index data
    let (vertex_data, index_data) = {
        let mut vertex_data = Vec::new();
        let mut index_data = Vec::new();

        let _ = create_cube(Vec3::ZERO, 10, &mut index_data, &mut vertex_data);

        (vertex_data, index_data)
    };
    let vertex_buffer_size = (vertex_data.len() * std::mem::size_of::<Vertex>()) as u32;
    let index_buffer_size = (index_data.len() * std::mem::size_of::<i32>()) as u32;

    // Create our vertex and index buffers
    let vertex_buffer = unsafe {
        let desc = SDL_GPUBufferCreateInfo {
            usage: SDL_GPU_BUFFERUSAGE_VERTEX,
            size: vertex_buffer_size,
            props: 0,
        };
        let buffer = SDL_CreateGPUBuffer(device.0, &desc).ok()?;
        SdlGpuBuffer::new(buffer, device.0)
    };
    let index_buffer = unsafe {
        let desc = SDL_GPUBufferCreateInfo {
            usage: SDL_GPU_BUFFERUSAGE_INDEX,
            size: index_buffer_size,
            props: 0,
        };
        let buffer = SDL_CreateGPUBuffer(device.0, &desc).ok()?;
        SdlGpuBuffer::new(buffer, device.0)
    };

    // Create our transform data
    let mut camera_position = Vec3::new(0.0, 50.0, -50.0);
    let mut camera_target = Vec3::new(0.0, 0.0, 0.0);
    let mut local_transform;
    let mut current_rotation: f32 = 0.0;
    let rotation_speed = 32.0 / 1000.0;
    let transform_buffer_size = std::mem::size_of::<[f32; 16]>() as u32;

    // Create the transfer buffer
    let transfer_buffer_size = vertex_buffer_size + index_buffer_size;
    let transfer_buffer = unsafe {
        let desc = SDL_GPUTransferBufferCreateInfo {
            usage: SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
            size: transfer_buffer_size,
            props: 0,
        };
        let buffer = SDL_CreateGPUTransferBuffer(device.0, &desc).ok()?;
        SdlGpuTransferBuffer::new(buffer, device.0)
    };

    // Update our buffers
    unsafe {
        let command_buffer = SDL_AcquireGPUCommandBuffer(device.0).ok()?;
        let copy_pass = SDL_BeginGPUCopyPass(command_buffer).ok()?;

        // Compute transfer buffer offsets
        let vertex_start = 0;
        let vertex_end = vertex_start + vertex_buffer_size as usize;

        let index_start = vertex_end;
        let index_end = index_start + index_buffer_size as usize;

        // Populate our transfer buffer
        {
            let dest_ptr = SDL_MapGPUTransferBuffer(device.0, transfer_buffer.get(), false).ok()?;
            let dest_slice =
                std::slice::from_raw_parts_mut(dest_ptr as *mut u8, transfer_buffer_size as usize);

            let vertex_bytes = std::slice::from_raw_parts(
                vertex_data.as_ptr() as *const u8,
                vertex_buffer_size as usize,
            );
            let index_bytes = std::slice::from_raw_parts(
                index_data.as_ptr() as *const u8,
                index_buffer_size as usize,
            );

            dest_slice[vertex_start..vertex_end].copy_from_slice(vertex_bytes);
            dest_slice[index_start..index_end].copy_from_slice(index_bytes);

            SDL_UnmapGPUTransferBuffer(device.0, transfer_buffer.get());
        }

        // Copy to our buffers
        {
            let source = SDL_GPUTransferBufferLocation {
                transfer_buffer: transfer_buffer.get(),
                offset: vertex_start as u32,
            };
            let dest = SDL_GPUBufferRegion {
                buffer: vertex_buffer.get(),
                offset: 0,
                size: vertex_buffer_size,
            };
            SDL_UploadToGPUBuffer(copy_pass, &source, &dest, false);
        }

        {
            let source = SDL_GPUTransferBufferLocation {
                transfer_buffer: transfer_buffer.get(),
                offset: index_start as u32,
            };
            let dest = SDL_GPUBufferRegion {
                buffer: index_buffer.get(),
                offset: 0,
                size: index_buffer_size,
            };
            SDL_UploadToGPUBuffer(copy_pass, &source, &dest, false);
        }

        // Execute and wait for the copies
        SDL_EndGPUCopyPass(copy_pass);
        let fence = SDL_SubmitGPUCommandBufferAndAcquireFence(command_buffer).ok()?;

        SDL_WaitForGPUFences(device.0, true, [fence].as_ptr(), 1).ok()?;
        SDL_ReleaseGPUFence(device.0, fence);
    }

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
            vertex_shader: vertex_shader.get(),
            fragment_shader: fragment_shader.get(),
            vertex_input_state,
            primitive_type: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            rasterizer_state: SDL_GPURasterizerState {
                fill_mode: SDL_GPU_FILLMODE_FILL,
                cull_mode: SDL_GPU_CULLMODE_BACK,
                front_face: SDL_GPU_FRONTFACE_COUNTER_CLOCKWISE,
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
        let pipeline = SDL_CreateGPUGraphicsPipeline(device.0, &desc).ok()?;
        SdlGpuGraphicsPipeline::new(pipeline, device.0)
    };

    // Message pump
    let mut quit = false;
    let mut event = SDL_Event {
        padding: [0u8; 128],
    };
    let mut last_update = Instant::now();
    while !quit {
        while unsafe { SDL_PollEvent(&mut event) } {
            match unsafe { SDL_EventType(event.r#type) } {
                SDL_EVENT_QUIT => {
                    quit = true;
                    break;
                }
                SDL_EVENT_KEY_UP => match unsafe { event.key.key } {
                    SDLK_Q => {
                        camera_position += Vec3::new(5.0, 0.0, 0.0);
                        camera_target += Vec3::new(5.0, 0.0, 0.0);
                    }
                    SDLK_A => {
                        camera_position -= Vec3::new(5.0, 0.0, 0.0);
                        camera_target -= Vec3::new(5.0, 0.0, 0.0);
                    }

                    SDLK_W => {
                        camera_position += Vec3::new(0.0, 5.0, 0.0);
                        camera_target += Vec3::new(0.0, 5.0, 0.0);
                    }
                    SDLK_S => {
                        camera_position -= Vec3::new(0.0, 5.0, 0.0);
                        camera_target -= Vec3::new(0.0, 5.0, 0.0);
                    }

                    SDLK_E => {
                        camera_position += Vec3::new(0.0, 0.0, 5.0);
                        camera_target += Vec3::new(0.0, 0.0, 5.0);
                    }
                    SDLK_D => {
                        camera_position -= Vec3::new(0.0, 0.0, 5.0);
                        camera_target -= Vec3::new(0.0, 0.0, 5.0);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Update
        let current_update = Instant::now();
        let elapsed = current_update - last_update;
        last_update = current_update;

        current_rotation =
            (current_rotation + (rotation_speed * elapsed.as_millis() as f32)) % 360.0;
        local_transform = Mat4::from_rotation_y(current_rotation.to_radians());

        // Render
        unsafe {
            let command_buffer = SDL_AcquireGPUCommandBuffer(device.0).ok()?;

            // Acquire the next swapchain texture
            let mut render_target = std::ptr::null_mut();
            let mut render_target_width = 0;
            let mut render_target_height = 0;
            SDL_AcquireGPUSwapchainTexture(
                command_buffer,
                window.0,
                &mut render_target,
                &mut render_target_width,
                &mut render_target_height,
            )
            .ok()?;

            // Draw
            let target_info = SDL_GPUColorTargetInfo {
                texture: render_target,
                mip_level: 0,
                layer_or_depth_plane: 0,
                clear_color: SDL_FColor {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                },
                load_op: SDL_GPU_LOADOP_CLEAR,
                store_op: SDL_GPU_STOREOP_STORE,
                resolve_texture: std::ptr::null_mut(),
                resolve_mip_level: 0,
                resolve_layer: 0,
                cycle: false,
                cycle_resolve_texture: false,
                padding1: 0,
                padding2: 0,
            };
            let render_pass =
                SDL_BeginGPURenderPass(command_buffer, &target_info, 1, std::ptr::null()).ok()?;

            SDL_BindGPUGraphicsPipeline(render_pass, pipeline.get());
            let viewport = SDL_GPUViewport {
                x: 0.0,
                y: 0.0,
                w: render_target_width as f32,
                h: render_target_height as f32,
                min_depth: 0.0,
                max_depth: 0.0,
            };
            SDL_SetGPUViewport(render_pass, &viewport);
            let vertex_bindings = [SDL_GPUBufferBinding {
                buffer: vertex_buffer.get(),
                offset: 0,
            }];
            SDL_BindGPUVertexBuffers(
                render_pass,
                0,
                vertex_bindings.as_ptr(),
                vertex_bindings.len() as u32,
            );
            let index_binding = SDL_GPUBufferBinding {
                buffer: index_buffer.get(),
                offset: 0,
            };
            SDL_BindGPUIndexBuffer(render_pass, &index_binding, SDL_GPU_INDEXELEMENTSIZE_32BIT);
            let world_transform = compute_world_transform(
                camera_position,
                camera_target,
                render_target_width,
                render_target_height,
            );
            SDL_PushGPUVertexUniformData(
                command_buffer,
                0,
                &world_transform as *const _ as *const _,
                transform_buffer_size,
            );
            SDL_PushGPUVertexUniformData(
                command_buffer,
                1,
                &local_transform as *const _ as *const _,
                transform_buffer_size,
            );

            SDL_DrawGPUIndexedPrimitives(render_pass, index_data.len() as u32, 1, 0, 0, 0);

            // Submit
            SDL_EndGPURenderPass(render_pass);
            let fence = SDL_SubmitGPUCommandBufferAndAcquireFence(command_buffer).ok()?;
            SDL_WaitForGPUFences(device.0, true, [fence].as_ptr(), 1).ok()?;
            SDL_ReleaseGPUFence(device.0, fence);
        }
    }

    unsafe {
        SDL_ReleaseWindowFromGPUDevice(device.0, window.0);
    }

    Ok(())
}

fn main() -> SdlResult<()> {
    // Init SDL
    unsafe {
        SDL_Init(SDL_INIT_VIDEO).ok()?;
    }

    run()?;

    unsafe {
        SDL_Quit();
    }

    Ok(())
}

fn compute_world_transform(
    camera_position: Vec3,
    camera_target: Vec3,
    width: u32,
    height: u32,
) -> Mat4 {
    let projection = Mat4::perspective_rh(
        45.0_f32.to_radians(),
        width as f32 / height as f32,
        1.0,
        10000.0,
    );
    let facing = (camera_target - camera_position).normalize();
    let view = Mat4::look_to_rh(camera_position, facing, Vec3::new(0.0, -1.0, 0.0));
    let correction = Mat4::from_cols_array_2d(&[
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    correction * projection * view
}

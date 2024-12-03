use sdl3_sys::{
    gpu::{
        SDL_DestroyGPUDevice, SDL_GPUBuffer, SDL_GPUDevice, SDL_GPUGraphicsPipeline, SDL_GPUShader,
        SDL_GPUTransferBuffer, SDL_ReleaseGPUBuffer, SDL_ReleaseGPUGraphicsPipeline,
        SDL_ReleaseGPUShader, SDL_ReleaseGPUTransferBuffer,
    },
    video::{SDL_DestroyWindow, SDL_Window},
};

macro_rules! destroy_wrapper {
    ($new_ty:ident, $sdl_ty:ident, $sdl_destroy:ident) => {
        #[repr(transparent)]
        pub struct $new_ty(pub *mut $sdl_ty);

        impl Drop for $new_ty {
            fn drop(&mut self) {
                unsafe {
                    $sdl_destroy(self.0);
                }
            }
        }

        impl From<*mut $sdl_ty> for $new_ty {
            fn from(value: *mut $sdl_ty) -> Self {
                $new_ty(value)
            }
        }
    };
}

destroy_wrapper!(SdlWindow, SDL_Window, SDL_DestroyWindow);
destroy_wrapper!(SdlGpuDevice, SDL_GPUDevice, SDL_DestroyGPUDevice);

macro_rules! device_resource {
    ($new_ty:ident, $sdl_ty:ident, $sdl_destroy:ident) => {
        pub struct $new_ty {
            inner: *mut $sdl_ty,
            device: *mut SDL_GPUDevice,
        }

        impl Drop for $new_ty {
            fn drop(&mut self) {
                unsafe {
                    $sdl_destroy(self.device, self.inner);
                }
            }
        }

        impl $new_ty {
            pub fn new(value: *mut $sdl_ty, device: *mut SDL_GPUDevice) -> Self {
                Self {
                    inner: value,
                    device,
                }
            }

            pub fn get(&self) -> *mut $sdl_ty {
                self.inner
            }
        }
    };
}

device_resource!(SdlGpuShader, SDL_GPUShader, SDL_ReleaseGPUShader);
device_resource!(SdlGpuBuffer, SDL_GPUBuffer, SDL_ReleaseGPUBuffer);
device_resource!(
    SdlGpuTransferBuffer,
    SDL_GPUTransferBuffer,
    SDL_ReleaseGPUTransferBuffer
);
device_resource!(
    SdlGpuGraphicsPipeline,
    SDL_GPUGraphicsPipeline,
    SDL_ReleaseGPUGraphicsPipeline
);

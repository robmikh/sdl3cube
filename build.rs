use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    // Ensure folders
    let _ = std::fs::create_dir_all("data/generated/shaders");

    // Compile shaders
    let input_path = "data/shaders/shader.wgsl";
    println!("cargo::rerun-if-changed={}", input_path);
    let vertex_shader = Shader::new(
        input_path,
        "data/generated/shaders/vertex.spv",
        "vs_main".to_owned(),
        ShaderStage::Vertex,
    );
    let fragment_shader = Shader::new(
        input_path,
        "data/generated/shaders/fragment.spv",
        "fs_main".to_owned(),
        ShaderStage::Fragment,
    );

    let backend = if cfg!(feature = "vulkan") {
        Backend::Vulkan
    } else if cfg!(feature = "dx12") {
        Backend::DX12
    } else if cfg!(feature = "metal") {
        Backend::Metal
    } else {
        Backend::Vulkan
    };

    vertex_shader.compile_for_backend(backend);
    fragment_shader.compile_for_backend(backend);
}

struct Shader {
    wgsl_path: PathBuf,
    spv_path: PathBuf,
    entrypoint: String,
    stage: ShaderStage,
}

impl Shader {
    fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        wgsl_path: P1,
        output_path: P2,
        entrypoint: String,
        stage: ShaderStage,
    ) -> Self {
        let wgsl_path = wgsl_path.as_ref().to_owned();
        let spv_path = output_path.as_ref().to_owned();
        Self {
            wgsl_path,
            spv_path,
            entrypoint,
            stage,
        }
    }

    fn compile_to_spv(&self) {
        compile_shader(
            &self.wgsl_path,
            &self.spv_path,
            self.entrypoint.clone(),
            self.stage,
        );
    }

    fn compile_for_backend(&self, backend: Backend) {
        self.compile_to_spv();
        if backend == Backend::Vulkan {
            return;
        }
        run_shadercross(&self.spv_path, &self.entrypoint, self.stage, backend);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Backend {
    Vulkan,
    DX12,
    Metal,
}

#[derive(Copy, Clone)]
enum ShaderStage {
    Vertex,
    Fragment,
}

impl From<ShaderStage> for naga::ShaderStage {
    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => Self::Vertex,
            ShaderStage::Fragment => Self::Fragment,
        }
    }
}

fn read_and_validate_shader<P: AsRef<Path>>(
    input_path: P,
) -> (naga::Module, naga::valid::ModuleInfo) {
    let input_text = std::fs::read_to_string(input_path).unwrap();
    let module = naga::front::wgsl::parse_str(&input_text).unwrap();
    let info = {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator.validate(&module).unwrap()
    };
    (module, info)
}

fn compile_shader<P1: AsRef<Path>, P2: AsRef<Path>>(
    input_path: P1,
    output_path: P2,
    entrypoint: String,
    stage: ShaderStage,
) {
    let (module, info) = read_and_validate_shader(input_path);
    let pipeline_options = naga::back::spv::PipelineOptions {
        shader_stage: stage.into(),
        entry_point: entrypoint,
    };
    let options = naga::back::spv::Options {
        ..Default::default()
    };
    let spv =
        naga::back::spv::write_vec(&module, &info, &options, Some(&pipeline_options)).unwrap();
    let bytes = spv
        .iter()
        .fold(Vec::with_capacity(spv.len() * 4), |mut v, w| {
            v.extend_from_slice(&w.to_le_bytes());
            v
        });
    std::fs::write(output_path, &bytes).unwrap()
}

fn run_shadercross<P: AsRef<Path>>(
    input_path: P,
    entrypoint: &str,
    stage: ShaderStage,
    backend: Backend,
) {
    let input_path = input_path.as_ref();
    let extension = match backend {
        Backend::Vulkan => "spv",
        Backend::DX12 => "dxil",
        Backend::Metal => "msl",
    };
    let output_path = input_path.with_extension(extension);
    let target = match backend {
        Backend::Vulkan => "SPIRV",
        Backend::DX12 => "DXIL",
        Backend::Metal => "MSL",
    };
    let stage = match stage {
        ShaderStage::Vertex => "vertex",
        ShaderStage::Fragment => "fragment",
    };
    let mut command = Command::new("shadercross");
    command.args([
        input_path.to_str().unwrap(),
        "-s",
        "SPIRV",
        "-d",
        target,
        "-e",
        entrypoint,
        "-o",
        output_path.to_str().unwrap(),
        "-t",
        stage,
    ]);
    // This will fail if you have the version of dxc from the Windows SDK
    // on your PATH. Grab the latest from GitHub and make sure it comes first.
    let status = command.status().unwrap();
    assert!(status.success());
}

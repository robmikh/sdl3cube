fn main() {
    // Compile shaders
    let input_path = "data/shaders/shader.wgsl";
    compile_shader(
        input_path,
        "data/generated/shaders/vertex.spv",
        "vs_main".to_owned(),
        ShaderStage::Vertex,
    );
    compile_shader(
        input_path,
        "data/generated/shaders/fragment.spv",
        "fs_main".to_owned(),
        ShaderStage::Fragment,
    );
}

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

fn compile_shader(input_path: &str, output_path: &str, entrypoint: String, stage: ShaderStage) {
    let input_text = std::fs::read_to_string(input_path).unwrap();
    let module = naga::front::wgsl::parse_str(&input_text).unwrap();
    let info = {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator.validate(&module).unwrap()
    };
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

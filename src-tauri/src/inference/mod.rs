pub mod demucs;
pub mod chunk;

use std::sync::Arc;

pub fn get_env() -> Arc<ort::environment::Environment> {
    ort::environment::Environment::current()
        .expect("Failed to get ONNX Runtime environment")
}

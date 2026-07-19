use candle_core::Device;

/// Split a "owner/name" model identifier into separate parts.
pub fn split_model_id(id: &str) -> (&str, &str) {
    id.split_once('/').unwrap_or(("", id))
}

/// Select a compute device based on a user preference string.
///
/// Valid values: `"auto"`, `"cpu"`, `"metal"`, `"cuda"`.
/// In `auto` mode the order is: CUDA → Metal → CPU.
pub fn select_device(preference: &str) -> anyhow::Result<Device> {
    match preference {
        "cpu" => {
            println!("using CPU");
            Ok(Device::Cpu)
        }
        "metal" => {
            if candle_core::utils::metal_is_available() {
                println!("using Metal GPU");
                Ok(Device::new_metal(0)?)
            } else {
                anyhow::bail!("Metal not available on this system");
            }
        }
        "cuda" => {
            if candle_core::utils::cuda_is_available() {
                println!("using CUDA GPU");
                Ok(Device::new_cuda(0)?)
            } else {
                anyhow::bail!("CUDA not available on this system");
            }
        }
        "auto" | _ => {
            if candle_core::utils::cuda_is_available() {
                println!("using CUDA GPU");
                Ok(Device::new_cuda(0)?)
            } else if candle_core::utils::metal_is_available() {
                println!("using Metal GPU");
                Ok(Device::new_metal(0)?)
            } else {
                println!("no GPU available, using CPU");
                Ok(Device::Cpu)
            }
        }
    }
}

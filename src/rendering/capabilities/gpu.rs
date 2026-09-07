use crate::contracts::capabilities::{GraphicsCard, GraphicsProbe, GraphicsTier};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, Default)]
pub struct WgpuGraphicsProbe;

impl GraphicsProbe for WgpuGraphicsProbe {
    fn devices(&self) -> Vec<crate::contracts::capabilities::GraphicsDevice> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let mut devices = instance
            .enumerate_adapters(wgpu::Backends::VULKAN)
            .into_iter()
            .filter_map(|adapter| {
                if adapter.get_info().device_type == wgpu::DeviceType::Cpu {
                    return None;
                }
                Some(crate::contracts::capabilities::GraphicsDevice {
                    id: device_id(&adapter)?,
                    name: adapter.get_info().name,
                })
            })
            .collect::<Vec<_>>();
        devices.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
        devices.dedup_by(|a, b| a.id == b.id);
        devices
    }

    fn probe(&self) -> GraphicsCard {
        let started = std::time::Instant::now();
        let backends = wgpu::Backends::VULKAN;
        let instance =
            wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..Default::default() });
        let preference = wgpu::PowerPreference::from_env().unwrap_or(wgpu::PowerPreference::None);
        let selected = iced::futures::executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: preference,
                compatible_surface: None,
                force_fallback_adapter: false,
            },
        ))
        .ok();
        log::info!(
            "gpu tier probe: selected {preference:?} adapter in {} ms",
            started.elapsed().as_millis()
        );
        match selected {
            Some(adapter) => {
                let info = adapter.get_info();
                GraphicsCard { name: info.name, tier: classify(info.device_type) }
            }
            None => GraphicsCard { name: String::from("unknown"), tier: GraphicsTier::Other },
        }
    }
}

fn device_id(adapter: &wgpu::Adapter) -> Option<String> {
    let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;
    if hal.physical_device_capabilities().properties().api_version < ash::vk::API_VERSION_1_1 {
        return None;
    }
    let mut ids = ash::vk::PhysicalDeviceIDProperties::default();
    let mut properties = ash::vk::PhysicalDeviceProperties2::default().push_next(&mut ids);
    unsafe {
        hal.shared_instance()
            .raw_instance()
            .get_physical_device_properties2(hal.raw_physical_device(), &mut properties);
    }
    let mut id = String::from("uuid:");
    for byte in ids.device_uuid {
        let _ = write!(id, "{byte:02x}");
    }
    Some(id)
}

pub(super) fn classify(device_type: wgpu::DeviceType) -> GraphicsTier {
    match device_type {
        wgpu::DeviceType::DiscreteGpu => GraphicsTier::Discrete,
        wgpu::DeviceType::IntegratedGpu => GraphicsTier::Integrated,
        _ => GraphicsTier::Other,
    }
}

pub fn effective_lod(lod: f32, auto: bool, tier: GraphicsTier) -> f32 {
    if auto && tier == GraphicsTier::Discrete { 1.0 } else { lod }
}

use gfx_hal as hal;
use hal::adapter::{Adapter, PhysicalDevice};
use hal::queue::QueueFamily;
use hal::Instance;

pub fn init_device() -> (
    gfx_backend_vulkan::Instance,
    Adapter<gfx_backend_vulkan::Backend>,
    gfx_backend_vulkan::Device,
) {
    let instance: gfx_backend_vulkan::Instance =
        crate::Instance::create("gfx-rs quad", 1).expect("Failed to create an instance!");

    let adapter = {
        let mut adapters = instance.enumerate_adapters();
        for adapter in &adapters {
            println!("{:?}", adapter.info);
        }
        adapters.remove(0)
    };

    // Build a new device and associated command queues
    let family = adapter
        .queue_families
        .iter()
        .find(|family| {
            family.queue_type().supports_graphics() //surface.supports_queue_family(family) &&
        })
        .expect("No queue family supports presentation");

    let physical_device = &adapter.physical_device;
    let sparsely_bound = physical_device
        .features()
        .contains(hal::Features::SPARSE_BINDING | hal::Features::SPARSE_RESIDENCY_IMAGE_2D);
    let gpu = unsafe {
        physical_device
            .open(
                &[(family, &[1.0])],
                if sparsely_bound {
                    hal::Features::SPARSE_BINDING
                        | hal::Features::SPARSE_RESIDENCY_IMAGE_2D
                        | hal::Features::EXTERNAL_MEMORY
                } else {
                    hal::Features::empty() | hal::Features::EXTERNAL_MEMORY
                },
            )
            .unwrap()
    };

    let device = gpu.device;

    (instance, adapter, device)
}

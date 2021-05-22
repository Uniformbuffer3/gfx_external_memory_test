use gfx_hal as hal;
use hal::device::Device;

pub const width: u32 = 1024;
pub const height: u32 = 768;

pub enum Parameters {
    Image{
        //data: Vec<u8>,
        kind: hal::image::Kind,
        mip_levels: hal::image::Level,
        format: hal::format::Format,
        tiling: hal::image::Tiling,
        usage: hal::image::Usage,
        sparse: hal::memory::SparseFlags,
        view_caps: hal::image::ViewCapabilities,
    },
    Buffer{
        //data: Vec<u8>,
        buffer_usage: hal::buffer::Usage,
        buffer_flags: hal::memory::SparseFlags
    }
}

pub fn read_memory<T: Default>(
    device: &gfx_backend_vulkan::Device,
    memory: &mut <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Memory,
) -> T {
    // Gather data from the imported memory
    let mapping = match unsafe { device.map_memory(memory, hal::memory::Segment::ALL) } {
        Ok(pointer) => pointer,
        Err(err) => panic!("Failed to `map_memory`:{:#?}", err),
    };
    let mut data = T::default();
    let data_len = std::mem::size_of::<T>() as u64;
    unsafe {
        std::ptr::copy_nonoverlapping(
            mapping,
            std::slice::from_mut(&mut data).as_mut_ptr() as *mut u8,
            data_len as usize,
        )
    };
    unsafe { device.unmap_memory(memory) };
    data
}

pub fn write_memory<T: Default>(
    device: &gfx_backend_vulkan::Device,
    memory: &mut <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Memory,
    data: &T,
) {
    // Write data on the memory
    let data_len = std::mem::size_of::<T>() as u64;
    let mapping = unsafe {
        device
            .map_memory(memory, hal::memory::Segment::ALL)
            .unwrap()
    };
    unsafe {
        std::ptr::copy_nonoverlapping(
            std::slice::from_ref(data).as_ptr() as *const u8,
            mapping,
            data_len as usize,
        )
    };
    unsafe {
        device
            .flush_mapped_memory_ranges(std::iter::once((&*memory, hal::memory::Segment::ALL)))
            .unwrap()
    };
    unsafe { device.unmap_memory(memory) };
}

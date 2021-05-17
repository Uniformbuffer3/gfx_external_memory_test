use gfx_hal as hal;
use hal::adapter::{Adapter, PhysicalDevice};
use hal::device::Device;

#[derive(Debug)]
pub enum TestResult {
    Success,
    Failed,
    Unsupported
}

#[derive(Debug)]
pub struct Tests {
    pub create_exportable_memory: Option<TestResult>,
    pub export_memory: Option<TestResult>,
    pub import_memory: Option<TestResult>,
    pub data_check: Option<TestResult>,
}


pub fn create_exportable_buffer<T>(
    adapter: &Adapter<gfx_backend_vulkan::Backend>,
    device: &gfx_backend_vulkan::Device,

    external_memory_type: hal::external_memory::ExternalMemoryType,
    buffer_usage: hal::buffer::Usage,
    buffer_flags: hal::memory::SparseFlags,
) -> Option<(
    <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Buffer,
    <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Memory,
)> {
    let data_len = std::mem::size_of::<T>() as u64;

    let buffer_properties = adapter
        .physical_device
        .query_external_buffer_properties(buffer_usage, buffer_flags, external_memory_type.into())
        .unwrap();

    if !buffer_properties.is_importable() {
        return None;
    }

    // Buffer allocations
    let limits = adapter.physical_device.properties().limits;
    let non_coherent_alignment = limits.non_coherent_atom_size as u64;

    assert_ne!(data_len, 0);
    let padded_buffer_len =
        ((data_len + non_coherent_alignment - 1) / non_coherent_alignment) * non_coherent_alignment;

    let mut buffer = unsafe {
        device
            .create_external_buffer(
                external_memory_type.into(),
                buffer_usage,
                buffer_flags,
                padded_buffer_len,
            )
            .unwrap()
    };

    let dedicated_allocation = if buffer_properties.requires_dedicated_allocation() {
        Some(hal::external_memory::BufferOrImage::Buffer(&buffer))
    } else {
        None
    };
    let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };

    let memory_type = adapter
        .physical_device
        .memory_properties()
        .memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
            // type_mask is a bit field where each bit represents a memory type. If the bit is set
            // to 1 it means we can use that type for our buffer. So this code finds the first
            // memory type that has a `1` (or, is allowed), and is visible to the CPU.
            buffer_req.type_mask & (1 << id) != 0
                && mem_type
                    .properties
                    .contains(hal::memory::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();

    let memory = match unsafe {
        device
            .allocate_exportable_memory(
                external_memory_type.into(),
                dedicated_allocation,
                memory_type,
                buffer_req.size,
            )
    }{
        Ok(memory)=>memory,
        Err(err)=>{
            println!("{:#?}",err);
            device.wait_idle().unwrap();
            unsafe {device.destroy_buffer(buffer);}
            return None;
        }

    };

    match unsafe { device.bind_buffer_memory(&memory, 0, &mut buffer) }{
        Ok(_)=>{Some((buffer, memory))}
        Err(err)=>{println!("{:#?}",err);None}
    }
}

pub fn import_buffer_memory(
    adapter: &Adapter<gfx_backend_vulkan::Backend>,
    device: &gfx_backend_vulkan::Device,

    external_memory: hal::external_memory::ExternalMemory,
    buffer_usage: hal::buffer::Usage,
    buffer_flags: hal::memory::SparseFlags,
) -> Option<(
    <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Buffer,
    <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Memory,
)> {
    let buffer_properties = adapter
        .physical_device
        .query_external_buffer_properties(buffer_usage, buffer_flags, external_memory.get_type())
        .unwrap();

    let mut buffer = unsafe {
        device
            .create_external_buffer(
                external_memory.get_type().into(),
                buffer_usage,
                buffer_flags,
                external_memory.get_size(),
            )
            .unwrap()
    };

    let dedicated_allocation = if buffer_properties.requires_dedicated_allocation() {
        Some(hal::external_memory::BufferOrImage::Buffer(&buffer))
    } else {
        None
    };

    let memory_mask = match unsafe { device.get_external_memory_mask(&external_memory) }{
        Ok(memory_mask)=>memory_mask,
        Err(err)=>{
            println!("{:#?}",err);
            device.wait_idle().unwrap();
            unsafe {device.destroy_buffer(buffer);}
            return None;
        }
    };
    println!("{:#?}",memory_mask);
    let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };
    let memory_type_id = adapter
        .physical_device
        .memory_properties()
        .memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
            // type_mask is a bit field where each bit represents a memory type. If the bit is set
            // to 1 it means we can use that type for our buffer. So this code finds the first
            // memory type that has a `1` (or, is allowed), and is visible to the CPU.
            memory_mask & (1 << id) != 0
                && mem_type
                    .properties
                    .contains(hal::memory::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();


    println!("{:#?}",buffer_req);
    let external_memory = match external_memory {
        #[cfg(any(unix))]
        hal::external_memory::ExternalMemory::Fd(external_memory_fd)=>{
            let (external_memory_type,fd,_size) = external_memory_fd.into();
            let external_memory_fd: hal::external_memory::ExternalMemoryFd = (external_memory_type,fd,buffer_req.size).into();
            external_memory_fd.into()
        }
        #[cfg(any(windows))]
        hal::external_memory::ExternalMemory::Handle(external_memory_handle)=>{
            let (external_memory_type,handle,_size) = external_memory_handle.into();
            let external_memory_handle: hal::external_memory::ExternalMemoryHandle =(external_memory_type,handle,buffer_req.size).into();
            external_memory_handle.into()
        }
        hal::external_memory::ExternalMemory::Ptr(external_memory_ptr)=>{
            let (external_memory_type,ptr,_size) = external_memory_ptr.into();
            let external_memory_ptr: hal::external_memory::ExternalMemoryPtr =(external_memory_type,ptr,buffer_req.size).into();
            external_memory_ptr.into()
        }
    };

    let memory = match unsafe {
        device.import_external_memory(external_memory, dedicated_allocation, memory_type_id)
    } {
        Ok(memory) => memory,
        Err(err)=>{
            println!("{:#?}",err);
            device.wait_idle().unwrap();
            unsafe {device.destroy_buffer(buffer);}
            return None;
        }
    };

    match unsafe { device.bind_buffer_memory(&memory, 0, &mut buffer) }{
        Ok(_)=>{Some((buffer, memory))}
        Err(err)=>{println!("{:#?}",err);None}
    }
}

pub fn read_memory<T: Default>(device: &gfx_backend_vulkan::Device, memory: &mut <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Memory)->T{
    // Gather data from the imported memory
    let mapping =
        match unsafe { device.map_memory(memory, hal::memory::Segment::ALL) } {
            Ok(pointer) => pointer,
            Err(err) => panic!("{:#?}", err),
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

pub fn write_memory<T: Default>(device: &gfx_backend_vulkan::Device,memory: &mut <gfx_backend_vulkan::Backend as gfx_hal::Backend>::Memory, data: &T){
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

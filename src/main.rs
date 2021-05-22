mod init_device;

mod common;
pub use common::*;

use log::*;

use gfx_hal as hal;
use hal::adapter::{Adapter, PhysicalDevice};
use hal::device::Device;
use hal::Instance;
use hal::external_memory::Resource;
use hal::format::AsFormat;
use std::convert::TryInto;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct DataTest {
    data: u32,
    data2: u32,
    data3: u32,
}

pub enum TestResult {
    Success,
    Failed,
}

impl std::fmt::Debug for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => f.write_str(":heavy_check_mark:"),
            Self::Failed => f.write_str(":x:"),
        }
    }
}

pub struct Tests {
    pub name: String,
    pub create_allocate_external_resource: Option<TestResult>,
    pub export_memory: Option<TestResult>,
    pub import_external_resource: Option<TestResult>,
    pub data_check: Option<TestResult>,
}

impl std::fmt::Debug for Tests {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.name.clone() + "\n")).unwrap();

        f.write_str("create_allocate_external_resource:").unwrap();
        match &self.create_allocate_external_resource {
            Some(result) => result.fmt(f).unwrap(),
            None => f.write_str(":fast_forward:").unwrap(),
        }
        f.write_str("\n").unwrap();

        f.write_str("export_memory:").unwrap();
        match &self.export_memory {
            Some(result) => result.fmt(f).unwrap(),
            None => f.write_str(":fast_forward:").unwrap(),
        }
        f.write_str("\n").unwrap();

        f.write_str("import_external_resource:").unwrap();
        match &self.import_external_resource {
            Some(result) => result.fmt(f).unwrap(),
            None => f.write_str(":fast_forward:").unwrap(),
        }
        f.write_str("\n").unwrap();

        f.write_str("data_check:").unwrap();
        match &self.data_check {
            Some(result) => result.fmt(f).unwrap(),
            None => f.write_str(":fast_forward:").unwrap(),
        }
        f.write_str("\n").unwrap();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let (_instance, adapter, device) = init_device::init_device();
    run_tests(&adapter, &device);
}

pub fn run_tests(
    adapter: &Adapter<gfx_backend_vulkan::Backend>,
    device: &gfx_backend_vulkan::Device,
) {
/*
    let img_data = std::include_bytes!("../logo.png");

    let img = image::load(std::io::Cursor::new(&img_data[..]), image::ImageFormat::Png)
        .unwrap()
        .to_rgba8();
    let (width, height) = img.dimensions();
    let row_alignment_mask = limits.optimal_buffer_copy_pitch_alignment as u32 - 1;
    let image_stride = 4usize;
    let row_pitch = (width * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
    let upload_size = (height * row_pitch) as u64;
    let padded_upload_size = ((upload_size + non_coherent_alignment - 1)
        / non_coherent_alignment)
        * non_coherent_alignment;



    for y in 0..height as usize {
        let row = &(*img)[y * (width as usize) * image_stride
            ..(y + 1) * (width as usize) * image_stride];
        ptr::copy_nonoverlapping(
            row.as_ptr(),
            mapping.offset(y as isize * row_pitch as isize),
            width as usize * image_stride,
        );
    }
*/
    println!("Resource: Buffer");
    #[cfg(any(unix))]
    {
        println!(
            "{:#?}",
            run_test(
                "OPAQUE_FD".into(),
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::OpaqueFd.into(),
                Parameters::Buffer {
                    buffer_usage: hal::buffer::Usage::VERTEX,
                    buffer_flags: hal::memory::SparseFlags::empty()
                }
            )
        );
        println!(
            "{:#?}",
            run_test(
                "DMA_BUF".into(),
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::DmaBuf.into(),
                Parameters::Buffer {
                    buffer_usage: hal::buffer::Usage::VERTEX,
                    buffer_flags: hal::memory::SparseFlags::empty()
                }
            )
        );
    }

    println!(
        "{:#?}",
        run_test(
            "HOST_ALLOCATION".into(),
            adapter,
            device,
            hal::external_memory::ExternalMemoryType::HostAllocation.into(),
                Parameters::Buffer {
                    buffer_usage: hal::buffer::Usage::VERTEX,
                    buffer_flags: hal::memory::SparseFlags::empty()
                }
        )
    );

    println!(
        "{:#?}",
        run_test(
            "HOST_MAPPED_FOREIGN_MEMORY".into(),
            adapter,
            device,
            hal::external_memory::ExternalMemoryType::HostMappedForeignMemory.into(),
                Parameters::Buffer {
                    buffer_usage: hal::buffer::Usage::VERTEX,
                    buffer_flags: hal::memory::SparseFlags::empty()
                }
        )
    );


    println!("Resource: Image");
    #[cfg(any(unix))]
    {
        println!(
            "{:#?}",
            run_test(
                "OPAQUE_FD".into(),
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::OpaqueFd.into(),
                Parameters::Image {
                    kind: hal::image::Kind::D2(width as hal::image::Size, height as hal::image::Size, 1, 1),
                    mip_levels: 1,
                    format: hal::format::Rgba8Srgb::SELF,
                    tiling: hal::image::Tiling::Linear,
                    usage: hal::image::Usage::TRANSFER_DST | hal::image::Usage::SAMPLED,
                    sparse: hal::memory::SparseFlags::empty(),
                    view_caps: hal::image::ViewCapabilities::empty(),
                }
            )
        );
        println!(
            "{:#?}",
            run_test(
                "DMA_BUF".into(),
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::DmaBuf.into(),
                Parameters::Image {
                    kind: hal::image::Kind::D2(width as hal::image::Size, height as hal::image::Size, 1, 1),
                    mip_levels: 1,
                    format: hal::format::Rgba8Srgb::SELF,
                    tiling: hal::image::Tiling::Linear,
                    usage: hal::image::Usage::TRANSFER_DST | hal::image::Usage::SAMPLED,
                    sparse: hal::memory::SparseFlags::empty(),
                    view_caps: hal::image::ViewCapabilities::empty(),
                }
            )
        );
    }

    println!(
        "{:#?}",
        run_test(
            "HOST_ALLOCATION".into(),
            adapter,
            device,
            hal::external_memory::ExternalMemoryType::HostAllocation.into(),
            Parameters::Image {
                kind: hal::image::Kind::D2(width as hal::image::Size, height as hal::image::Size, 1, 1),
                mip_levels: 1,
                format: hal::format::Rgba8Srgb::SELF,
                tiling: hal::image::Tiling::Linear,
                usage: hal::image::Usage::TRANSFER_DST | hal::image::Usage::SAMPLED,
                sparse: hal::memory::SparseFlags::empty(),
                view_caps: hal::image::ViewCapabilities::empty(),
            }
        )
    );

    println!(
        "{:#?}",
        run_test(
            "HOST_MAPPED_FOREIGN_MEMORY".into(),
            adapter,
            device,
            hal::external_memory::ExternalMemoryType::HostMappedForeignMemory.into(),
            Parameters::Image {
                kind: hal::image::Kind::D2(width as hal::image::Size, height as hal::image::Size, 1, 1),
                mip_levels: 1,
                format: hal::format::Rgba8Srgb::SELF,
                tiling: hal::image::Tiling::Linear,
                usage: hal::image::Usage::TRANSFER_DST | hal::image::Usage::SAMPLED,
                sparse: hal::memory::SparseFlags::empty(),
                view_caps: hal::image::ViewCapabilities::empty(),
            }
        )
    );
}

pub fn run_test(
    name: String,

    adapter: &Adapter<gfx_backend_vulkan::Backend>,
    device: &gfx_backend_vulkan::Device,

    external_memory_type: hal::external_memory::ExternalMemoryType,
    parameters: Parameters,
) -> Tests {

    let mut tests = Tests {
        name: name,
        create_allocate_external_resource: None,
        export_memory: None,
        import_external_resource: None,
        data_check: None,
    };

    let external_memory_properties = match parameters {
        Parameters::Buffer{buffer_usage,buffer_flags}=>{
            match adapter
            .physical_device
            .query_external_buffer_properties(buffer_usage, buffer_flags, external_memory_type.into())
            {
                Ok(external_memory_properties)=>external_memory_properties,
                Err(err)=>{
                    error!("Error on `query_external_buffer_properties`: {:#?}",err);
                    return tests;
                }
            }
        }
        Parameters::Image{kind: _,mip_levels: _,format,tiling,usage,sparse: _,view_caps}=>{
            match adapter
            .physical_device
            .query_external_image_properties(format,2,tiling,usage,view_caps, external_memory_type.into())
            {
                Ok(external_memory_properties)=>external_memory_properties,
                Err(err)=>{
                    error!("Error on `query_external_image_properties`: {:#?}",err);
                    return tests;
                }
            }
        }
    };

    println!("{:#?}",&external_memory_properties);




    let memory_types: u32 = adapter
        .physical_device
        .memory_properties()
        .memory_types
        .into_iter()
        .enumerate()
        .map(|(id, mem_type)| {
            if mem_type
                .properties
                .contains(hal::memory::Properties::CPU_VISIBLE)
            {
                1 << id
            } else {
                0
            }
        })
        .sum();

    // Buffer allocations
    let data_in = crate::DataTest::default();
    let data_len = std::mem::size_of::<crate::DataTest>() as u64;
    let physical_device_properties = adapter.physical_device.properties();
    //let non_coherent_alignment = physical_device_properties.limits.non_coherent_atom_size as u64;
    let host_ptr_alignment = physical_device_properties.external_memory_limits.min_imported_host_pointer_alignment;

    assert_ne!(data_len, 0);
    let padded_buffer_len =
        ((data_len + host_ptr_alignment - 1) / host_ptr_alignment) * host_ptr_alignment;

    match (
        external_memory_properties.is_exportable(),
        external_memory_properties.is_importable(),
        external_memory_properties.is_exportable_from_imported(),
    ) {
        (true, false, _) => {
            let (resource,mut memory): (Resource<gfx_backend_vulkan::Backend>,_) = match parameters {
                Parameters::Buffer{buffer_usage,buffer_flags}=>{
                    let (buffer, memory) = match unsafe {
                        device.create_allocate_external_buffer(
                            external_memory_type.into(),
                            buffer_usage,
                            buffer_flags,
                            memory_types,
                            padded_buffer_len,
                        )
                    } {
                        Ok(buffer_memory) => {
                            tests.create_allocate_external_resource = Some(TestResult::Success);
                            buffer_memory
                        }
                        Err(err) => {
                            error!("Error on `create_allocate_external_resource`: {:#?}", err);
                            tests.create_allocate_external_resource = Some(TestResult::Failed);
                            return tests;
                        }
                    };
                    (Resource::Buffer(buffer),memory)
                }
                Parameters::Image{kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
                    let (image, memory) = match unsafe {
                        device.create_allocate_external_image(
                            external_memory_type.into(),
                            kind,mip_levels,format,tiling,usage,sparse,view_caps,
                            memory_types
                        )
                    } {
                        Ok(image_memory) => {
                            tests.create_allocate_external_resource = Some(TestResult::Success);
                            image_memory
                        }
                        Err(err) => {
                            error!("Error on `create_allocate_external_resource`: {:#?}", err);
                            tests.create_allocate_external_resource = Some(TestResult::Failed);
                            return tests;
                        }
                    };
                    (Resource::Image(image),memory)
                }
            };

            write_memory(device, &mut memory, &data_in);

            let _external_memory = if external_memory_type
                == hal::external_memory::ExternalMemoryType::HostAllocation
                || external_memory_type
                    == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
            {
                match unsafe { device.map_memory(&mut memory, hal::memory::Segment::ALL) } {
                    Ok(external_memory) => {
                        tests.export_memory = Some(TestResult::Success);
                        let ptr: hal::external_memory::Ptr = external_memory.into();
                        (external_memory_type, ptr).try_into().unwrap()
                    }
                    Err(err) => {
                        error!("Error on `export_memory`: {:#?}", err);
                        tests.export_memory = Some(TestResult::Failed);
                        device.wait_idle().unwrap();
                        unsafe {
                            match resource {
                                Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                Resource::Image(image)=>device.destroy_image(image)
                            }
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            } else {
                match unsafe { device.export_memory(external_memory_type, &memory) } {
                    Ok(external_memory) => {
                        tests.export_memory = Some(TestResult::Success);
                        external_memory
                    }
                    Err(err) => {
                        error!("Error on `export_memory`: {:#?}", err);
                        tests.export_memory = Some(TestResult::Failed);
                        device.wait_idle().unwrap();
                        unsafe {
                            match resource {
                                Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                Resource::Image(image)=>device.destroy_image(image)
                            }
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            };
            device.wait_idle().unwrap();
            unsafe {
                device.wait_idle().unwrap();
                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                    device.unmap_memory(&mut memory);
                }
                match resource {
                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                    Resource::Image(image)=>device.destroy_image(image)
                }
                device.free_memory(memory);
            }
        }
        (true, true, true) => {
            let (resource,mut memory): (Resource<gfx_backend_vulkan::Backend>,_) = match parameters {
                Parameters::Buffer{buffer_usage,buffer_flags}=>{
                    let (buffer, memory) = match unsafe {
                        device.create_allocate_external_buffer(
                            external_memory_type.into(),
                            buffer_usage,
                            buffer_flags,
                            memory_types,
                            padded_buffer_len,
                        )
                    } {
                        Ok(buffer_memory) => {
                            tests.create_allocate_external_resource = Some(TestResult::Success);
                            buffer_memory
                        }
                        Err(err) => {
                            error!("Error on `create_allocate_external_resource`: {:#?}", err);
                            tests.create_allocate_external_resource = Some(TestResult::Failed);
                            return tests;
                        }
                    };
                    (Resource::Buffer(buffer),memory)
                }
                Parameters::Image{kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
                    let (image, memory) = match unsafe {
                        device.create_allocate_external_image(
                            external_memory_type.into(),
                            kind,mip_levels,format,tiling,usage,sparse,view_caps,
                            memory_types
                        )
                    } {
                        Ok(image_memory) => {
                            tests.create_allocate_external_resource = Some(TestResult::Success);
                            image_memory
                        }
                        Err(err) => {
                            error!("Error on `create_allocate_external_resource`: {:#?}", err);
                            tests.create_allocate_external_resource = Some(TestResult::Failed);
                            return tests;
                        }
                    };
                    (Resource::Image(image),memory)
                }
            };

            write_memory(device, &mut memory, &data_in);

            let external_memory = if external_memory_type
                == hal::external_memory::ExternalMemoryType::HostAllocation
                || external_memory_type
                    == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
            {
                match unsafe { device.map_memory(&mut memory, hal::memory::Segment::ALL) } {
                    Ok(external_memory) => {
                        tests.export_memory = Some(TestResult::Success);
                        let ptr: hal::external_memory::Ptr = external_memory.into();
                        (external_memory_type, ptr).try_into().unwrap()
                    }
                    Err(err) => {
                        error!("Error on `export_memory`: {:#?}", err);
                        tests.export_memory = Some(TestResult::Failed);
                        device.wait_idle().unwrap();
                        unsafe {
                            match resource {
                                Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                Resource::Image(image)=>device.destroy_image(image)
                            }
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            } else {
                match unsafe { device.export_memory(external_memory_type, &memory) } {
                    Ok(external_memory) => {
                        tests.export_memory = Some(TestResult::Success);
                        external_memory
                    }
                    Err(err) => {
                        error!("Error on `export_memory`: {:#?}", err);
                        tests.export_memory = Some(TestResult::Failed);
                        device.wait_idle().unwrap();
                        unsafe {
                        match resource {
                            Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                            Resource::Image(image)=>device.destroy_image(image)
                        }
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            };
/*
            let (imported_buffer, mut imported_memory) = match unsafe {
                device.import_external_resource(
                    external_memory,
                    buffer_usage,
                    buffer_flags,
                    memory_types.clone(),
                    padded_buffer_len,
                )
            } {
                Ok(buffer_memory) => {
                    tests.import_external_resource = Some(TestResult::Success);
                    buffer_memory
                }
                Err(err) => {
                    error!("Error on `import_external_resource`: {:#?}", err);
                    tests.import_external_resource = Some(TestResult::Failed);
                    device.wait_idle().unwrap();
                    unsafe {
                        if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                        external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                            device.unmap_memory(&mut memory);
                        }
                        match resource {
                            Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                            Resource::Image(image)=>device.destroy_image(image)
                        }
                        device.free_memory(memory);
                    }
                    return tests;
                }
            };
*/
            let (imported_resource,mut imported_memory): (Resource<gfx_backend_vulkan::Backend>,_) = match parameters {
                Parameters::Buffer{buffer_usage,buffer_flags}=>{
                    let (buffer, memory) = match unsafe {
                        device.import_external_buffer(
                            external_memory,
                            buffer_usage,
                            buffer_flags,
                            memory_types.clone(),
                            padded_buffer_len,
                        )
                    } {
                        Ok(buffer_memory) => {
                            tests.import_external_resource = Some(TestResult::Success);
                            buffer_memory
                        }
                        Err(err) => {
                            error!("Error on `import_external_resource`: {:#?}", err);
                            tests.import_external_resource = Some(TestResult::Failed);
                            device.wait_idle().unwrap();
                            unsafe {
                                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                                    device.unmap_memory(&mut memory);
                                }
                                match resource {
                                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                    Resource::Image(image)=>device.destroy_image(image)
                                }
                                device.free_memory(memory);
                            }
                            return tests;
                        }
                    };
                    (Resource::Buffer(buffer),memory)
                }
                Parameters::Image{kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
                    let (image, memory) = match unsafe {
                        device.import_external_image(
                            external_memory,
                            kind,mip_levels,format,tiling,usage,sparse,view_caps,
                            memory_types.clone()
                        )
                    } {
                        Ok(image_memory) => {
                            tests.import_external_resource = Some(TestResult::Success);
                            image_memory
                        }
                        Err(err) => {
                            error!("Error on `import_external_resource`: {:#?}", err);
                            tests.import_external_resource = Some(TestResult::Failed);
                            device.wait_idle().unwrap();
                            unsafe {
                                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                                    device.unmap_memory(&mut memory);
                                }
                                match resource {
                                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                    Resource::Image(image)=>device.destroy_image(image)
                                }
                                device.free_memory(memory);
                            }
                            return tests;
                        }
                    };
                    (Resource::Image(image),memory)
                }
            };

            let data_out = read_memory::<crate::DataTest>(device, &mut imported_memory);
            if data_in == data_out {
                tests.data_check = Some(TestResult::Success);
            } else {
                tests.data_check = Some(TestResult::Failed);
            }

            device.wait_idle().unwrap();
            unsafe {
                match imported_resource {
                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                    Resource::Image(image)=>device.destroy_image(image)
                }
                device.free_memory(imported_memory);
                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                    device.unmap_memory(&mut memory);
                }
                match resource {
                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                    Resource::Image(image)=>device.destroy_image(image)
                }
                device.free_memory(memory);
            }
        }
        (false,true,false)=>{
            if hal::external_memory::ExternalMemoryType::HostAllocation == external_memory_type
                || hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
                    == external_memory_type
            {
            } else {
                return tests;
            }

            let (resource,mut memory): (Resource<gfx_backend_vulkan::Backend>,_)= match parameters {
                Parameters::Buffer{buffer_usage,buffer_flags}=>{
                    let (buffer, memory) = match unsafe {
                        device.create_allocate_external_buffer(
                            external_memory_type.into(),
                            buffer_usage,
                            buffer_flags,
                            memory_types,
                            padded_buffer_len,
                        )
                    } {
                        Ok(buffer_memory) => {
                            tests.create_allocate_external_resource = Some(TestResult::Success);
                            buffer_memory
                        }
                        Err(err) => {
                            error!("Error on `create_allocate_external_resource`: {:#?}", err);
                            tests.create_allocate_external_resource = Some(TestResult::Failed);
                            return tests;
                        }
                    };
                    (Resource::Buffer(buffer),memory)
                }
                Parameters::Image{kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
                    let (image, memory) = match unsafe {
                        device.create_allocate_external_image(
                            external_memory_type.into(),
                            kind,mip_levels,format,tiling,usage,sparse,view_caps,
                            memory_types
                        )
                    } {
                        Ok(image_memory) => {
                            tests.create_allocate_external_resource = Some(TestResult::Success);
                            image_memory
                        }
                        Err(err) => {
                            error!("Error on `create_allocate_external_resource`: {:#?}", err);
                            tests.create_allocate_external_resource = Some(TestResult::Failed);
                            return tests;
                        }
                    };
                    (Resource::Image(image),memory)
                }
            };

            write_memory(device, &mut memory, &data_in);

            let external_memory = if external_memory_type
                == hal::external_memory::ExternalMemoryType::HostAllocation
                || external_memory_type
                    == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
            {
                match unsafe { device.map_memory(&mut memory, hal::memory::Segment::ALL) } {
                    Ok(external_memory) => {
                        tests.export_memory = Some(TestResult::Success);
                        let ptr: hal::external_memory::Ptr = external_memory.into();
                        (external_memory_type, ptr).try_into().unwrap()
                    }
                    Err(err) => {
                        error!("Error on `export_memory`: {:#?}", err);
                        tests.export_memory = Some(TestResult::Failed);
                        device.wait_idle().unwrap();
                        unsafe {
                            match resource {
                                Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                Resource::Image(image)=>device.destroy_image(image)
                            }
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            } else {
                match unsafe { device.export_memory(external_memory_type, &memory) } {
                    Ok(external_memory) => {
                        tests.export_memory = Some(TestResult::Success);
                        external_memory
                    }
                    Err(err) => {
                        error!("Error on `export_memory`: {:#?}", err);
                        tests.export_memory = Some(TestResult::Failed);
                        device.wait_idle().unwrap();
                        unsafe {
                            match resource {
                                Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                Resource::Image(image)=>device.destroy_image(image)
                            }
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            };
            let (imported_resource,mut imported_memory): (Resource<gfx_backend_vulkan::Backend>,_) = match parameters {
                Parameters::Buffer{buffer_usage,buffer_flags}=>{
                    let (buffer, memory) = match unsafe {
                        device.import_external_buffer(
                            external_memory,
                            buffer_usage,
                            buffer_flags,
                            memory_types.clone(),
                            padded_buffer_len,
                        )
                    } {
                        Ok(buffer_memory) => {
                            tests.import_external_resource = Some(TestResult::Success);
                            buffer_memory
                        }
                        Err(err) => {
                            error!("Error on `import_external_resource`: {:#?}", err);
                            tests.import_external_resource = Some(TestResult::Failed);
                            device.wait_idle().unwrap();
                            unsafe {
                                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                                    device.unmap_memory(&mut memory);
                                }
                                match resource {
                                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                    Resource::Image(image)=>device.destroy_image(image)
                                }
                                device.free_memory(memory);
                            }
                            return tests;
                        }
                    };
                    (Resource::Buffer(buffer),memory)
                }
                Parameters::Image{kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
                    let (image, memory) = match unsafe {
                        device.import_external_image(
                            external_memory,
                            kind,mip_levels,format,tiling,usage,sparse,view_caps,
                            memory_types.clone()
                        )
                    } {
                        Ok(image_memory) => {
                            tests.import_external_resource = Some(TestResult::Success);
                            image_memory
                        }
                        Err(err) => {
                            error!("Error on `import_external_resource`: {:#?}", err);
                            tests.import_external_resource = Some(TestResult::Failed);
                            device.wait_idle().unwrap();
                            unsafe {
                                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                                    device.unmap_memory(&mut memory);
                                }
                                match resource {
                                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                                    Resource::Image(image)=>device.destroy_image(image)
                                }
                                device.free_memory(memory);
                            }
                            return tests;
                        }
                    };
                    (Resource::Image(image),memory)
                }
            };

            let data_out = read_memory::<crate::DataTest>(device, &mut imported_memory);
            if data_in == data_out {
                tests.data_check = Some(TestResult::Success);
            } else {
                tests.data_check = Some(TestResult::Failed);
            }

            device.wait_idle().unwrap();
            unsafe {
                match imported_resource {
                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                    Resource::Image(image)=>device.destroy_image(image)
                }
                device.free_memory(imported_memory);
                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                    device.unmap_memory(&mut memory);
                }
                match resource {
                    Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                    Resource::Image(image)=>device.destroy_image(image)
                }
                device.free_memory(memory);
            }

        }
        _ => {}
    }
    return tests;
}

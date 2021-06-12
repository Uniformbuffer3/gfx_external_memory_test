mod init_device;

mod common;
pub use common::*;


use log::*;

use gfx_hal as hal;
use hal::adapter::{Adapter, PhysicalDevice};
use hal::device::Device;
use hal::Instance;
use hal::format::AsFormat;
use std::convert::TryInto;
use hal::external_memory::*;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

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
                Parameters::Buffer {
                    external_memory_type: hal::external_memory::ExternalBufferMemoryType::OpaqueFd,
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
                Parameters::Buffer {
                    external_memory_type: hal::external_memory::ExternalBufferMemoryType::DmaBuf,
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
            Parameters::Buffer {
                external_memory_type: hal::external_memory::ExternalBufferMemoryType::HostAllocation,
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
            Parameters::Buffer {
                external_memory_type: hal::external_memory::ExternalBufferMemoryType::HostMappedForeignMemory,
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
                Parameters::Image {
                    external_memory_type: hal::external_memory::ExternalImageMemoryType::OpaqueFd,
                    kind: hal::image::Kind::D2(WIDTH as hal::image::Size, HEIGHT as hal::image::Size, 1, 1),
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
                Parameters::Image {
                    external_memory_type: hal::external_memory::ExternalImageMemoryType::DmaBuf(Vec::new()),
                    kind: hal::image::Kind::D2(WIDTH as hal::image::Size, HEIGHT as hal::image::Size, 1, 1),
                    mip_levels: 1,
                    format: hal::format::Rgba8Srgb::SELF,
                    tiling: hal::image::Tiling::Linear,
                    usage: hal::image::Usage::TRANSFER_DST | hal::image::Usage::SAMPLED,
                    sparse: hal::memory::SparseFlags::empty(),
                    view_caps: hal::image::ViewCapabilities::empty(),
                }
            )
        );

        /*
        let format_properties = adapter.physical_device.format_properties(Some(hal::format::Rgba8Srgb::SELF));
        println!("{:#?}",format_properties);
        println!(
            "{:#?}",
            run_test(
                "DMA_BUF".into(),
                adapter,
                device,
                Parameters::Image {
                    external_memory_type: hal::external_memory::ExternalImageMemoryType::DmaBuf(Vec::new()),
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
        */
    }

    println!(
        "{:#?}",
        run_test(
            "HOST_ALLOCATION".into(),
            adapter,
            device,
            Parameters::Image {
                external_memory_type: hal::external_memory::ExternalImageMemoryType::HostAllocation,
                kind: hal::image::Kind::D2(WIDTH as hal::image::Size, HEIGHT as hal::image::Size, 1, 1),
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

            Parameters::Image {
                external_memory_type: hal::external_memory::ExternalImageMemoryType::HostMappedForeignMemory,
                kind: hal::image::Kind::D2(WIDTH as hal::image::Size, HEIGHT as hal::image::Size, 1, 1),
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

    parameters: Parameters,
) -> Tests {
    let mut tests = Tests {
        name: name,
        create_allocate_external_resource: None,
        export_memory: None,
        import_external_resource: None,
        data_check: None,
    };

    let external_memory_properties = match parameters.clone() {
        Parameters::Buffer{external_memory_type,buffer_usage,buffer_flags}=>{
            adapter
            .physical_device
            .external_buffer_properties(buffer_usage, buffer_flags, external_memory_type)
        }
        Parameters::Image{external_memory_type, kind: _,mip_levels: _,format,tiling,usage,sparse: _,view_caps}=>{
            match adapter
            .physical_device
            .external_image_properties(format,2,tiling,usage,view_caps, external_memory_type.external_memory_type())
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

    let external_memory_type = match parameters.clone() {
        Parameters::Image{external_memory_type,..}=>external_memory_type.external_memory_type(),
        Parameters::Buffer{external_memory_type,..}=>external_memory_type
    };


    let mut exportable_resource = None;
    let mut exportable_memory = None;
    let mut exported_memory = None;

    let mut imported_resource = None;
    let mut imported_memory = None;

    if external_memory_properties.contains(hal::external_memory::ExternalMemoryProperties::EXPORTABLE){
        let (resource,mut memory): (Resource<gfx_backend_vulkan::Backend>,_) = match parameters.clone() {
            Parameters::Buffer{external_memory_type,buffer_usage,buffer_flags}=>{
                let (buffer, memory) = match unsafe {
                    device.create_allocate_external_buffer(
                        external_memory_type,
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
            Parameters::Image{external_memory_type,kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
                let (image, memory) = match unsafe {
                    device.create_allocate_external_image(
                        external_memory_type,
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
                    hal::external_memory::PlatformMemory::Ptr(ptr)
                }
                Err(err) => {
                    error!("Error on `export_memory`: {:#?}", err);
                    tests.export_memory = Some(TestResult::Failed);
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
                    return tests;
                }
            }
        };

        exportable_resource = Some(resource);
        exportable_memory = Some(memory);
        exported_memory = Some(external_memory);
    }

    if external_memory_properties.contains(hal::external_memory::ExternalMemoryProperties::IMPORTABLE) && exported_memory.is_some() {
        let exported_memory = exported_memory.unwrap();
        let (resource,mut memory): (Resource<gfx_backend_vulkan::Backend>,_) = match parameters.clone() {
            Parameters::Buffer{external_memory_type,buffer_usage,buffer_flags}=>{
                let external_memory = match external_memory_type {
                    #[cfg(unix)]
                    ExternalMemoryType::OpaqueFd => ExternalBufferMemory::OpaqueFd(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalMemoryType::OpaqueWin32 => ExternalBufferMemory::OpaqueWin32(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalMemoryType::OpaqueWin32Kmt => ExternalBufferMemory::OpaqueWin32Kmt(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalMemoryType::D3D11Texture => ExternalBufferMemory::D3D11Texture(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalMemoryType::D3D11TextureKmt => ExternalBufferMemory::D3D11TextureKmt(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalMemoryType::D3D12Heap => ExternalBufferMemory::D3D12Heap(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalMemoryType::D3D12Resource => ExternalBufferMemory::D3D12Resource(exported_memory.try_into().unwrap()),
                    #[cfg(any(target_os = "linux", target_os = "android", doc))]
                    ExternalMemoryType::DmaBuf => ExternalBufferMemory::DmaBuf(exported_memory.try_into().unwrap()),
                    #[cfg(any(target_os = "android", doc))]
                    ExternalMemoryType::AndroidHardwareBuffer => ExternalBufferMemory::AndroidHardwareBuffer(exported_memory.try_into().unwrap()),
                    ExternalMemoryType::HostAllocation => ExternalBufferMemory::HostAllocation(exported_memory.try_into().unwrap()),
                    ExternalMemoryType::HostMappedForeignMemory => ExternalBufferMemory::HostMappedForeignMemory(exported_memory.try_into().unwrap()),
                };

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
                        return tests;
                    }
                };
                (Resource::Buffer(buffer),memory)
            }
            Parameters::Image{external_memory_type,kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
                let external_memory = match external_memory_type {
                    #[cfg(unix)]
                    ExternalImageMemoryType::OpaqueFd => ExternalImageMemory::OpaqueFd(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalImageMemoryType::OpaqueWin32 => ExternalImageMemory::OpaqueWin32(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalImageMemoryType::OpaqueWin32Kmt => ExternalImageMemory::OpaqueWin32Kmt(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalImageMemoryType::D3D11Texture => ExternalImageMemory::D3D11Texture(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalImageMemoryType::D3D11TextureKmt => ExternalImageMemory::D3D11TextureKmt(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalImageMemoryType::D3D12Heap => ExternalImageMemory::D3D12Heap(exported_memory.try_into().unwrap()),
                    #[cfg(windows)]
                    ExternalImageMemoryType::D3D12Resource => ExternalImageMemory::D3D12Resource(exported_memory.try_into().unwrap()),
                    #[cfg(any(target_os = "linux", target_os = "android", doc))]
                    ExternalImageMemoryType::DmaBuf(_)=> ExternalImageMemory::DmaBuf(exported_memory.try_into().unwrap(),None),
                    #[cfg(any(target_os = "android", doc))]
                    ExternalImageMemoryType::AndroidHardwareBuffer => ExternalImageMemory::AndroidHardwareBuffer(exported_memory.try_into().unwrap()),
                    ExternalImageMemoryType::HostAllocation => ExternalImageMemory::HostAllocation(exported_memory.try_into().unwrap()),
                    ExternalImageMemoryType::HostMappedForeignMemory => ExternalImageMemory::HostMappedForeignMemory(exported_memory.try_into().unwrap()),
                };
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
                        return tests;
                    }
                };
                (Resource::Image(image),memory)
            }
        };

        let data_out = read_memory::<crate::DataTest>(device, &mut memory);
        if data_in == data_out {
            tests.data_check = Some(TestResult::Success);
        } else {
            tests.data_check = Some(TestResult::Failed);
        }

        imported_resource = Some(resource);
        imported_memory = Some(memory);
    }


    if external_memory_properties.contains(hal::external_memory::ExternalMemoryProperties::EXPORTABLE_FROM_IMPORTED) && imported_memory.is_some() {
        if external_memory_type
            == hal::external_memory::ExternalMemoryType::HostAllocation
            || external_memory_type
                == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
        {
            match unsafe { device.map_memory(imported_memory.as_mut().unwrap(), hal::memory::Segment::ALL) } {
                Ok(_external_memory) => {
                    tests.export_memory = Some(TestResult::Success);
                }
                Err(err) => {
                    error!("Error on `export_memory`: {:#?}", err);
                    tests.export_memory = Some(TestResult::Failed);
                    return tests;
                }
            }
        } else {
            match unsafe { device.export_memory(external_memory_type, imported_memory.as_ref().unwrap()) } {
                Ok(_external_memory) => {
                    tests.export_memory = Some(TestResult::Success);
                }
                Err(err) => {
                    error!("Error on `export_memory`: {:#?}", err);
                    tests.export_memory = Some(TestResult::Failed);
                    return tests;
                }
            }
        }
    }

    device.wait_idle().unwrap();

    unsafe {
        if let Some(resource) = exportable_resource {
            match resource {
                Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                Resource::Image(image)=>device.destroy_image(image)
            }
        }

        if let Some(memory) = exportable_memory {
            device.free_memory(memory);
        }

        if let Some(resource) = imported_resource {
            match resource {
                Resource::Buffer(buffer)=>device.destroy_buffer(buffer),
                Resource::Image(image)=>device.destroy_image(image)
            }
        }

        if let Some(memory) = imported_memory {
            device.free_memory(memory);
        }
    }

    return tests;
}

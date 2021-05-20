mod init_device;

mod common;
pub use common::*;

use log::*;

use gfx_hal as hal;
use hal::adapter::{Adapter, PhysicalDevice};
use hal::device::Device;
use hal::Instance;

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
    pub create_allocate_external_buffer: Option<TestResult>,
    pub export_memory: Option<TestResult>,
    pub import_external_buffer: Option<TestResult>,
    pub data_check: Option<TestResult>,
}

impl std::fmt::Debug for Tests {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.name.clone() + "\n")).unwrap();

        f.write_str("create_allocate_external_buffer:").unwrap();
        match &self.create_allocate_external_buffer {
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

        f.write_str("import_external_buffer:").unwrap();
        match &self.import_external_buffer {
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
    #[cfg(any(unix))]
    {
        println!(
            "{:#?}",
            run_test(
                "OPAQUE_FD".into(),
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::OpaqueFd.into(),
                hal::buffer::Usage::VERTEX,
                hal::memory::SparseFlags::empty(),
            )
        );
        println!(
            "{:#?}",
            run_test(
                "DMA_BUF".into(),
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::DmaBuf.into(),
                hal::buffer::Usage::VERTEX,
                hal::memory::SparseFlags::empty(),
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
            hal::buffer::Usage::VERTEX,
            hal::memory::SparseFlags::empty(),
        )
    );

    println!(
        "{:#?}",
        run_test(
            "HOST_MAPPED_FOREIGN_MEMORY".into(),
            adapter,
            device,
            hal::external_memory::ExternalMemoryType::HostMappedForeignMemory.into(),
            hal::buffer::Usage::VERTEX,
            hal::memory::SparseFlags::empty(),
        )
    );
}

pub fn run_test(
    name: String,

    adapter: &Adapter<gfx_backend_vulkan::Backend>,
    device: &gfx_backend_vulkan::Device,

    external_memory_type: hal::external_memory::ExternalMemoryType,
    buffer_usage: hal::buffer::Usage,
    buffer_flags: hal::memory::SparseFlags,
) -> Tests {
    let buffer_properties = adapter
        .physical_device
        .query_external_buffer_properties(buffer_usage, buffer_flags, external_memory_type.into())
        .unwrap();

    let mut tests = Tests {
        name: name,
        create_allocate_external_buffer: None,
        export_memory: None,
        import_external_buffer: None,
        data_check: None,
    };

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
    let limits = adapter.physical_device.properties().limits;
    let non_coherent_alignment = limits.non_coherent_atom_size as u64;

    assert_ne!(data_len, 0);
    let padded_buffer_len =
        ((data_len + non_coherent_alignment - 1) / non_coherent_alignment) * non_coherent_alignment;

    match (
        buffer_properties.is_exportable(),
        buffer_properties.is_importable(),
        buffer_properties.is_exportable_from_imported(),
    ) {
        (true, false, _) => {
            let (buffer, mut memory) = match unsafe {
                device.create_allocate_external_buffer(
                    external_memory_type.into(),
                    buffer_usage,
                    buffer_flags,
                    memory_types.clone(),
                    padded_buffer_len,
                )
            } {
                Ok(buffer_memory) => {
                    tests.create_allocate_external_buffer = Some(TestResult::Success);
                    buffer_memory
                }
                Err(err) => {
                    error!("Error on `create_allocate_external_buffer`: {:#?}", err);
                    tests.create_allocate_external_buffer = Some(TestResult::Failed);
                    return tests;
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
                            device.destroy_buffer(buffer);
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
                            device.destroy_buffer(buffer);
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            };
            device.wait_idle().unwrap();
            unsafe {
                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                    device.unmap_memory(&mut memory);
                }
                device.destroy_buffer(buffer);
                device.free_memory(memory);
            }
        }

        (true, true, true) => {
            let (buffer, mut memory) = match unsafe {
                device.create_allocate_external_buffer(
                    external_memory_type.into(),
                    buffer_usage,
                    buffer_flags,
                    memory_types.clone(),
                    padded_buffer_len,
                )
            } {
                Ok(buffer_memory) => {
                    tests.create_allocate_external_buffer = Some(TestResult::Success);
                    buffer_memory
                }
                Err(err) => {
                    error!("Error on `create_allocate_external_buffer`: {:#?}", err);
                    tests.create_allocate_external_buffer = Some(TestResult::Failed);
                    return tests;
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
                            device.destroy_buffer(buffer);
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
                            device.destroy_buffer(buffer);
                            device.free_memory(memory);
                        }
                        return tests;
                    }
                }
            };

            let (imported_buffer, mut imported_memory) = match unsafe {
                device.import_external_buffer(
                    external_memory,
                    buffer_usage,
                    buffer_flags,
                    memory_types.clone(),
                    data_len,
                )
            } {
                Ok(buffer_memory) => {
                    tests.import_external_buffer = Some(TestResult::Success);
                    buffer_memory
                }
                Err(err) => {
                    error!("Error on `import_external_buffer`: {:#?}", err);
                    tests.import_external_buffer = Some(TestResult::Failed);
                    device.wait_idle().unwrap();
                    unsafe {
                        if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                        external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                            device.unmap_memory(&mut memory);
                        }

                        device.destroy_buffer(buffer);
                        device.free_memory(memory);
                    }
                    return tests;
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
                device.destroy_buffer(imported_buffer);
                device.free_memory(imported_memory);

                if external_memory_type == hal::external_memory::ExternalMemoryType::HostAllocation ||
                external_memory_type == hal::external_memory::ExternalMemoryType::HostMappedForeignMemory {
                    device.unmap_memory(&mut memory);
                }

                device.destroy_buffer(buffer);
                device.free_memory(memory);
            }
        }

        (false, true, false) => {
            if hal::external_memory::ExternalMemoryType::HostAllocation == external_memory_type
                || hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
                    == external_memory_type
            {
            } else {
                return tests;
            }

            let (buffer, mut memory) = match unsafe {
                device.create_allocate_external_buffer(
                    external_memory_type.into(),
                    buffer_usage,
                    buffer_flags,
                    memory_types.clone(),
                    padded_buffer_len,
                )
            } {
                Ok(buffer_memory) => {
                    tests.create_allocate_external_buffer = Some(TestResult::Success);
                    buffer_memory
                }
                Err(err) => {
                    error!("Error on `create_allocate_external_buffer`: {:#?}", err);
                    tests.create_allocate_external_buffer = Some(TestResult::Failed);
                    return tests;
                }
            };

            write_memory(device, &mut memory, &data_in);

            let (imported_buffer, mut imported_memory) = match external_memory_type {
                hal::external_memory::ExternalMemoryType::HostAllocation
                | hal::external_memory::ExternalMemoryType::HostMappedForeignMemory => {
                    match unsafe { device.map_memory(&mut memory, hal::memory::Segment::ALL) } {
                        Ok(ptr) => {
                            tests.export_memory = Some(TestResult::Success);

                            match unsafe {
                                device.import_external_buffer(
                                    (external_memory_type, hal::external_memory::Ptr::from(ptr))
                                        .try_into()
                                        .unwrap(),
                                    buffer_usage,
                                    buffer_flags,
                                    memory_types.clone(),
                                    data_len,
                                )
                            } {
                                Ok(buffer_memory) => {
                                    tests.import_external_buffer = Some(TestResult::Success);
                                    buffer_memory
                                }
                                Err(err) => {
                                    error!("Error on `import_external_buffer`: {:#?}", err);
                                    tests.import_external_buffer = Some(TestResult::Failed);
                                    device.wait_idle().unwrap();
                                    unsafe {
                                        device.destroy_buffer(buffer);
                                        device.free_memory(memory);
                                    }
                                    return tests;
                                }
                            }
                        }
                        Err(err) => {
                            error!("Error on `map_memory`: {:#?}", err);
                            tests.export_memory = Some(TestResult::Failed);
                            device.wait_idle().unwrap();
                            unsafe {
                                device.destroy_buffer(buffer);
                                device.free_memory(memory);
                            }
                            return tests;
                        }
                    }
                }
                _ => {
                    unimplemented!()
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
                device.destroy_buffer(imported_buffer);
                device.free_memory(imported_memory);

                if hal::external_memory::ExternalMemoryType::HostAllocation == external_memory_type
                    || hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
                        == external_memory_type
                {
                    device.unmap_memory(&mut memory);
                }

                device.destroy_buffer(buffer);
                device.free_memory(memory);
            }
        }
        _ => {}
    }
    return tests;
}

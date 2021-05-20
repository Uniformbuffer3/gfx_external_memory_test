mod init_device;

mod common;
pub use common::*;

use log::*;

use gfx_hal as hal;
use hal::adapter::{Adapter, PhysicalDevice};
use hal::device::Device;
use hal::Instance;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct DataTest {
    data: u32,
    data2: u32,
    data3: u32,
}

#[derive(Debug)]
pub enum TestResult {
    Success,
    Failed,
    Unsupported,
}

#[derive(Debug)]
pub struct Tests {
    pub create_allocate_external_buffer: Option<TestResult>,
    pub export_memory: Option<TestResult>,
    pub import_external_buffer: Option<TestResult>,
    pub data_check: Option<TestResult>,
}

fn main() {
    env_logger::init();
    let (_instance, adapter, device) = init_device::init_device();

    println!("{:#?}", adapter.physical_device.memory_properties());

    run_tests(&adapter, &device);
}

pub fn run_tests(
    adapter: &Adapter<gfx_backend_vulkan::Backend>,
    device: &gfx_backend_vulkan::Device,
) {
    #[cfg(any(unix))]
    {
        println!(
            "OPAQUE_FD:\n{:#?}",
            run_test(
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::OpaqueFd.into(),
                hal::buffer::Usage::VERTEX,
                hal::memory::SparseFlags::empty(),
            )
        );
        println!(
            "DMA_BUF:\n{:#?}",
            run_test(
                adapter,
                device,
                hal::external_memory::ExternalMemoryType::DmaBuf.into(),
                hal::buffer::Usage::VERTEX,
                hal::memory::SparseFlags::empty(),
            )
        );
    }

    println!(
        "HOST_ALLOCATION:\n{:#?}",
        run_test(
            adapter,
            device,
            hal::external_memory::ExternalMemoryType::HostAllocation.into(),
            hal::buffer::Usage::VERTEX,
            hal::memory::SparseFlags::empty(),
        )
    );

    println!(
        "HOST_MAPPED_FOREIGN_MEMORY:\n{:#?}",
        run_test(
            adapter,
            device,
            hal::external_memory::ExternalMemoryType::HostMappedForeignMemory.into(),
            hal::buffer::Usage::VERTEX,
            hal::memory::SparseFlags::empty(),
        )
    );
}

pub fn run_test(
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
        create_allocate_external_buffer: None,
        export_memory: None,
        import_external_buffer: None,
        data_check: None,
    };

    //println!("{:#?}",&buffer_properties);

    let memory_types: Vec<hal::MemoryTypeId> = adapter
        .physical_device
        .memory_properties()
        .memory_types
        .into_iter()
        .enumerate()
        .filter_map(|(id, mem_type)| {
            if (1 << id) != 0
                && mem_type
                    .properties
                    .contains(hal::memory::Properties::CPU_VISIBLE)
            {
                Some(id.into())
            } else {
                None
            }
        })
        .collect();

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
        buffer_properties
            .get_export_from_imported_memory_types()
            .contains(external_memory_type.into()),
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

            let _external_memory =
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
                };

            device.wait_idle().unwrap();
            unsafe {
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

            let external_memory =
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

                //if let hal::external_memory::ExternalMemoryType::Ptr(_) = external_memory_type {device.unmap_memory(&mut memory);}

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
                    use std::convert::TryInto;
                    match unsafe { device.map_memory(&mut memory, hal::memory::Segment::ALL) } {
                        Ok(ptr) => {
                            tests.export_memory = Some(TestResult::Success);

                            match unsafe {
                                device.import_external_buffer(
                                    (
                                        external_memory_type,
                                        hal::external_memory::Ptr::from(ptr).into(),
                                    )
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

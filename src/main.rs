mod init_device;

mod common;
pub use common::*;

use gfx_hal as hal;
use hal::Instance;
use hal::adapter::{Adapter, PhysicalDevice};
use hal::device::Device;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct DataTest {
    data: u32,
    data2: u32,
    data3: u32,
}

fn main() {
    env_logger::init();
    let (_instance, adapter, device) = init_device::init_device();

    println!("{:#?}",adapter.physical_device.memory_properties());

    run_tests(&adapter, &device);
}



pub fn run_tests(
    adapter: &Adapter<gfx_backend_vulkan::Backend>,
    device: &gfx_backend_vulkan::Device,
) {
    #[cfg(any(unix))]
    {
    println!("OPAQUE_FD:\n{:#?}",run_test(
        adapter,
        device,
        hal::external_memory::ExternalMemoryFdType::OPAQUE_FD.into(),
        hal::buffer::Usage::VERTEX,
        hal::memory::SparseFlags::empty(),
    ));

    println!("DMA_BUF:\n{:#?}",run_test(
        adapter,
        device,
        hal::external_memory::ExternalMemoryFdType::DMA_BUF.into(),
        hal::buffer::Usage::VERTEX,
        hal::memory::SparseFlags::empty(),
    ));
    }

    println!("HOST_ALLOCATION:\n{:#?}",run_test(
        adapter,
        device,
        hal::external_memory::ExternalMemoryPtrType::HOST_ALLOCATION.into(),
        hal::buffer::Usage::VERTEX,
        hal::memory::SparseFlags::empty(),
    ));

    println!("HOST_MAPPED_FOREIGN_MEMORY:\n{:#?}",run_test(
        adapter,
        device,
        hal::external_memory::ExternalMemoryPtrType::HOST_MAPPED_FOREIGN_MEMORY.into(),
        hal::buffer::Usage::VERTEX,
        hal::memory::SparseFlags::empty(),
    ));
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
        create_exportable_memory: None,
        export_memory: None,
        import_memory: None,
        data_check: None
    };

    //println!("{:#?}",&buffer_properties);

    match (buffer_properties.is_exportable(),buffer_properties.is_importable(), buffer_properties.get_export_from_imported_memory_types().contains(external_memory_type.into()))
    {
        (true,false,_)=>{
            let (buffer, mut memory) = match create_exportable_buffer::<crate::DataTest>(
                adapter,
                device,
                external_memory_type.into(),
                buffer_usage,
                buffer_flags,
            )
            {
                Some(buffer_memory)=>{tests.create_exportable_memory = Some(TestResult::Success);buffer_memory}
                None=>{
                    //error!("Failed to create exportable buffer: {:#?}",err);
                    tests.create_exportable_memory = Some(TestResult::Failed);
                    return tests;
                }
            };

            let data_in = crate::DataTest::default();
            write_memory(device,&mut memory,&data_in);

            match external_memory_type {
                #[cfg(any(unix))]
                hal::external_memory::ExternalMemoryType::Fd(external_memory_fd_type)=>{
                    match unsafe { device.export_memory_as_fd(external_memory_fd_type, &memory) } {
                        Ok(_)=>{tests.export_memory = Some(TestResult::Success)},
                        Err(_)=>{tests.export_memory = Some(TestResult::Failed);}
                    }

                }
                #[cfg(any(windows))]
                hal::external_memory::ExternalMemoryType::Handle(external_memory_handle_type)=>{
                    match unsafe { device.export_memory_as_handle(external_memory_handle_type, &memory) }{
                        Ok(_)=>{Some(ResultTest::Success)}
                        Err(err)=>{tests.export_memory = Some(ResultTest::Failed);}
                    }
                }
                _=>{tests.export_memory = Some(TestResult::Unsupported);}
            }
            device.wait_idle().unwrap();
            unsafe {
                device.destroy_buffer(buffer);
                device.free_memory(memory);
            }
        }
        (true,true,true)=>{
            let (buffer, mut memory) = match create_exportable_buffer::<crate::DataTest>(
                adapter,
                device,
                external_memory_type.into(),
                buffer_usage,
                buffer_flags,
            )
            {
                Some(buffer_memory)=>{tests.create_exportable_memory = Some(TestResult::Success);buffer_memory}
                None=>{
                    tests.create_exportable_memory = Some(TestResult::Failed);
                    return tests;
                }
            };

            let data_in = crate::DataTest::default();
            //let data_len = std::mem::size_of::<crate::DataTest>() as u64;
            write_memory(device,&mut memory,&data_in);
            let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };

            let (imported_buffer, mut imported_memory) = match external_memory_type {
                #[cfg(any(unix))]
                hal::external_memory::ExternalMemoryType::Fd(external_memory_fd_type)=>{
                    match unsafe { device.export_memory_as_fd(external_memory_fd_type, &memory) } {
                        Ok(fd)=>{
                            tests.export_memory = Some(TestResult::Success);
                            let external_memory: hal::external_memory::ExternalMemoryFd = (external_memory_fd_type,fd.into(), buffer_req.size).into();
                            match import_buffer_memory(
                                adapter,
                                device,
                                external_memory.into(),
                                buffer_usage,
                                buffer_flags
                            )
                            {
                                Some(buffer_memory)=>{tests.import_memory = Some(TestResult::Success);buffer_memory}
                                None=>{
                                    tests.import_memory = Some(TestResult::Failed);
                                    device.wait_idle().unwrap();
                                    unsafe {
                                        device.destroy_buffer(buffer);
                                        device.free_memory(memory);
                                    }
                                    return tests;
                                }
                            }
                        },
                        Err(_)=>{
                            //error!("Failed to export buffer as fd: {:#?}",err);
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
                #[cfg(any(windows))]
                hal::external_memory::ExternalMemoryType::Handle(external_memory_handle_type)=>{
                    match unsafe { device.export_memory_as_handle(external_memory_handle_type, &memory) } {
                        Ok(handle)=>{
                            tests.export_memory = Some(TestResult::Success);
                            let external_memory: hal::external_memory::ExternalMemoryHandle = (external_memory_handle_type,handle.into(), data_len).into();
                            match crate::tests::common::import_buffer_memory(
                                adapter,
                                device,
                                external_memory.into(),
                                buffer_usage,
                                buffer_flags
                            )
                            {
                                Some(buffer_memory)=>{tests.import_memory = Some(TestResult::Success);buffer_memory}
                                None=>{
                                    tests.import_memory = Some(TestResult::Failed);
                                    device.wait_idle().unwrap();
                                    unsafe {
                                        device.destroy_buffer(buffer);
                                        device.free_memory(memory);
                                    }
                                    return tests;
                                }
                            }
                        },
                        Err(err)=>{
                            tests.export_memory = Some(TestResult::Failed);
                            device.wait_idle().unwrap();
                            unsafe {
                                device.destroy_buffer(buffer);
                                device.free_memory(memory);
                            }
                            return tests;
                        }
                    };
                }
                hal::external_memory::ExternalMemoryType::Ptr(external_memory_ptr_type)=>{
                    match unsafe { device.map_memory(&mut memory, hal::memory::Segment::ALL) } {
                        Ok(ptr)=>{
                            tests.export_memory = Some(TestResult::Success);
                            let external_memory: hal::external_memory::ExternalMemoryPtr = (external_memory_ptr_type,ptr.into(), buffer_req.size).into();
                            match import_buffer_memory(
                                adapter,
                                device,
                                external_memory.into(),
                                buffer_usage,
                                buffer_flags
                            )
                            {
                                Some(buffer_memory)=>{tests.import_memory = Some(TestResult::Success);buffer_memory}
                                None=>{
                                    tests.import_memory = Some(TestResult::Failed);
                                    device.wait_idle().unwrap();
                                    unsafe {
                                        device.destroy_buffer(buffer);
                                        device.free_memory(memory);
                                    }
                                    return tests;
                                }
                            }
                        },
                        Err(_)=>{
                            //error!("Failed to export buffer as fd: {:#?}",err);
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
            };

            let data_out = read_memory::<crate::DataTest>(device,&mut imported_memory);
            if data_in == data_out {tests.data_check = Some(TestResult::Success);}
            else {tests.data_check = Some(TestResult::Failed);}

            device.wait_idle().unwrap();
            unsafe {
                device.destroy_buffer(imported_buffer);
                device.free_memory(imported_memory);

                if let hal::external_memory::ExternalMemoryType::Ptr(_) = external_memory_type {device.unmap_memory(&mut memory);}

                device.destroy_buffer(buffer);
                device.free_memory(memory);
            }
        }

        (false,true,false)=>{
            if let hal::external_memory::ExternalMemoryType::Ptr(_) = external_memory_type {}
            else {return tests;}

            let (buffer, mut memory) = match create_exportable_buffer::<crate::DataTest>(
                adapter,
                device,
                external_memory_type.into(),
                buffer_usage,
                buffer_flags,
            )
            {
                Some(buffer_memory)=>{tests.create_exportable_memory = Some(TestResult::Success);buffer_memory}
                None=>{
                    tests.create_exportable_memory = Some(TestResult::Failed);
                    return tests;
                }
            };

            let data_in = crate::DataTest::default();
            write_memory(device,&mut memory,&data_in);
            let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };

            let (imported_buffer, mut imported_memory) = match external_memory_type {
                #[cfg(any(unix))]
                hal::external_memory::ExternalMemoryType::Fd(_)=>{
                    unimplemented!();
                }
                #[cfg(any(windows))]
                hal::external_memory::ExternalMemoryType::Handle(_)=>{
                    unimplemented!();
                }
                hal::external_memory::ExternalMemoryType::Ptr(external_memory_ptr_type)=>{
                    match unsafe { device.map_memory(&mut memory, hal::memory::Segment::ALL) } {
                        Ok(ptr)=>{
                            tests.export_memory = Some(TestResult::Success);
                            let external_memory: hal::external_memory::ExternalMemoryPtr = (external_memory_ptr_type,ptr.into(), buffer_req.size).into();
                            match import_buffer_memory(
                                adapter,
                                device,
                                external_memory.into(),
                                buffer_usage,
                                buffer_flags
                            )
                            {
                                Some(buffer_memory)=>{tests.import_memory = Some(TestResult::Success);buffer_memory}
                                None=>{
                                    tests.import_memory = Some(TestResult::Failed);
                                    device.wait_idle().unwrap();
                                    unsafe {
                                        device.destroy_buffer(buffer);
                                        device.free_memory(memory);
                                    }
                                    return tests;
                                }
                            }
                        },
                        Err(_)=>{
                            //error!("Failed to export buffer as fd: {:#?}",err);
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
            };

            let data_out = read_memory::<crate::DataTest>(device,&mut imported_memory);
            if data_in == data_out {tests.data_check = Some(TestResult::Success);}
            else {tests.data_check = Some(TestResult::Failed);}

            device.wait_idle().unwrap();
            unsafe {
                device.destroy_buffer(imported_buffer);
                device.free_memory(imported_memory);

                if let hal::external_memory::ExternalMemoryType::Ptr(_) = external_memory_type {device.unmap_memory(&mut memory);}

                device.destroy_buffer(buffer);
                device.free_memory(memory);
            }
        }
        _=>{}
    }
    return tests;
}

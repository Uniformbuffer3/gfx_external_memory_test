
/*
            let mut buffer = unsafe {device.create_buffer(padded_buffer_len,buffer_usage,buffer_flags)}.unwrap();
            let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };
            let memory_type = (0..32).into_iter()
                .find(|id| {
                    // type_mask is a bit field where each bit represents a memory type. If the bit is set
                    // to 1 it means we can use that type for our buffer. So this code finds the first
                    // memory type that has a `1` (or, is allowed), and is visible to the CPU.
                    buffer_req.type_mask & (1 << id) & memory_types != 0
                })
                .unwrap()
                .into();
            let mut memory = unsafe {
                let memory = device
                    .allocate_memory(memory_type, buffer_req.size)
                    .unwrap();
                device
                    .bind_buffer_memory(&memory, 0, &mut buffer)
                    .unwrap();
                memory
            };
*/


/*
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
*/


/*
    match (
        external_memory_properties.contains(hal::external_memory::ExternalMemoryProperties::EXPORTABLE),
        external_memory_properties.contains(hal::external_memory::ExternalMemoryProperties::IMPORTABLE),
        external_memory_properties.contains(hal::external_memory::ExternalMemoryProperties::EXPORTABLE_FROM_IMPORTED),
    ) {

        (true, false, _) => {
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

            let external_memory_type = match parameters {
                Parameters::Image{external_memory_type,..}=>external_memory_type.external_memory_type(),
                Parameters::Buffer{external_memory_type,..}=>external_memory_type
            };

            let _external_memory = if external_memory_type
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
                Parameters::Buffer{external_memory_type,buffer_usage,buffer_flags}=>{
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
                Parameters::Image{external_memory_type,kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
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

            let external_memory_type = match parameters {
                Parameters::Image{external_memory_type,..}=>external_memory_type.external_memory_type(),
                Parameters::Buffer{external_memory_type,..}=>external_memory_type
            };

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
                Parameters::Buffer{external_memory_type,buffer_usage,buffer_flags}=>{
                    let external_memory = match external_memory_type {
                        #[cfg(unix)]
                        ExternalMemoryType::OpaqueFd => PlatformMemoryType::Fd,
                        #[cfg(windows)]
                        ExternalMemoryType::OpaqueWin32 => PlatformMemoryType::Handle,
                        #[cfg(windows)]
                        ExternalMemoryType::OpaqueWin32Kmt => PlatformMemoryType::Handle,
                        #[cfg(windows)]
                        ExternalMemoryType::D3D11Texture => PlatformMemoryType::Handle,
                        #[cfg(windows)]
                        ExternalMemoryType::D3D11TextureKmt => PlatformMemoryType::Handle,
                        #[cfg(windows)]
                        ExternalMemoryType::D3D12Heap => PlatformMemoryType::Handle,
                        #[cfg(windows)]
                        ExternalMemoryType::D3D12Resource => PlatformMemoryType::Handle,
                        #[cfg(any(target_os = "linux", target_os = "android", doc))]
                        ExternalMemoryType::DmaBuf => PlatformMemoryType::Fd,
                        #[cfg(any(target_os = "android", doc))]
                        ExternalMemoryType::AndroidHardwareBuffer => PlatformMemoryType::Fd,
                        ExternalMemoryType::HostAllocation => PlatformMemoryType::Ptr,
                        ExternalMemoryType::HostMappedForeignMemory => PlatformMemoryType::Ptr,
                    }
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
                Parameters::Image{external_memory_type,kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
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
                                if external_memory_type == hal::external_memory::ExternalImageMemoryType::HostAllocation ||
                                external_memory_type == hal::external_memory::ExternalImageMemoryType::HostMappedForeignMemory {
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
        /*
        (false,true,false)=>{
            if hal::external_memory::ExternalMemoryType::HostAllocation == external_memory_type
                || hal::external_memory::ExternalMemoryType::HostMappedForeignMemory
                    == external_memory_type
            {
            } else {
                return tests;
            }

            let (resource,mut memory): (Resource<gfx_backend_vulkan::Backend>,_)= match parameters {
                Parameters::Buffer{external_memory_type,buffer_usage,buffer_flags}=>{
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
                Parameters::Image{external_memory_type,kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
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
                        hal::external_memory::ExternalMemory::from_platform(external_memory_type, ptr.into()).unwrap()
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
                Parameters::Buffer{external_memory_type,buffer_usage,buffer_flags}=>{
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
                Parameters::Image{external_memory_type,kind,mip_levels,format,tiling,usage,sparse,view_caps}=>{
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
                                if external_memory_type == hal::external_memory::ExternalImageMemoryType::HostAllocation ||
                                external_memory_type == hal::external_memory::ExternalImageMemoryType::HostMappedForeignMemory {
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

        }*/
        _ => {}
    }
*/

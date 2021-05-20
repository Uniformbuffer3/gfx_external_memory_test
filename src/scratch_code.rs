
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

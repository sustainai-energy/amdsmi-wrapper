// Copyright 2026 Sustain AI
// Licensed under the Apache License, Version 2.0 (the "License");
// SPDX-License-Identifier: Apache-2.0

//! Rust wrapper for AMD SMI (System Management Interface)
//!
//! This crate provides a high-level, safe interface to the amdsmi library for managing AMD devices.
//!
//! # Example
//!
//! ```ignore
//! use amdsmi_wrapper::AmdSmi;
//!
//! let amdsmi = AmdSmi::init()?;
//! let entity = amdsmi.get_entity_by_bdf("0000:03:00.0")?;
//! entity.set_...;
//! ```

pub mod error;
pub mod types;

use crate::types::{
    Bdf, EngineUsage, Frequencies, PerfLevel, PowerCapInfo, PowerCapSensors, PowerConsumption,
};
use amdsmi_wrapper_sys::{
    amdsmi_clk_limit_type_t, amdsmi_clk_type_t, amdsmi_engine_usage_t, amdsmi_frequencies_t,
    amdsmi_init_flags_t, amdsmi_power_cap_type_t, amdsmi_power_info_t, amdsmi_processor_handle,
    amdsmi_status_t, libamd_smi,
};
use error::AmdSmiError;
use std::ffi::CStr;
use std::mem::MaybeUninit;

/// Main AMD SMI interface
///
/// This struct manages the amdsmi library initialization and provides methods to discover and
/// interact with AMD GPU devices.
pub struct AmdSmi {
    lib: libamd_smi,
}

impl AmdSmi {
    /// Initialize the AMD SMI library using `amdsmi_init`
    ///
    /// Currently only supports AMD GPUs (AMDSMI_INIT_AMD_GPUS flag).
    pub fn init() -> Result<Self, AmdSmiError> {
        // SAFETY: libamd_smi::new loads the system library "libamd_smi.so" at runtime.
        // This is safe because we're loading a standard system library with a hardcoded name.
        let lib = unsafe { libamd_smi::new("libamd_smi.so")? };
        // SAFETY: amdsmi_init is a standard initialization function (fn(u64) -> amdsmi_status_t).
        // We pass AMDSMI_INIT_AMD_GPUS which is a valid flag value from the amdsmi_init_flags_t enum.
        let status = unsafe {
            lib.amdsmi_init.as_ref().expect("amdsmi_init not loaded")(
                amdsmi_init_flags_t::AMDSMI_INIT_AMD_GPUS as u64,
            )
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(AmdSmi { lib })
        } else {
            Err(AmdSmiError::Status(status))
        }
    }

    /// Gets a entity handle directly from a BDF string (e.g., "0000:01:00.0").
    ///
    /// This is a convenience method that parses the BDF string and calls
    /// `get_processor_handle_from_bdf`.
    pub fn get_entity_by_bdf(&self, bdf_str: &str) -> Result<Entity<'_>, AmdSmiError> {
        let bdf: Bdf = bdf_str.parse()?;
        let mut handle = amdsmi_processor_handle::default();
        // SAFETY: amdsmi_get_processor_handle_from_bdf has signature
        // fn(bdf: amdsmi_bdf_t, processor_handle: *mut amdsmi_processor_handle) -> amdsmi_status_t.
        // We pass a valid BDF from parsing and a pointer to an uninitialized handle.
        // The function will write the handle to this pointer on success.
        let status = unsafe {
            self.lib
                .amdsmi_get_processor_handle_from_bdf
                .as_ref()
                .expect("amdsmi_get_processor_handle_from_bdf not loaded")(
                bdf.into(), &mut handle
            )
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(Entity {
                handle,
                lib: &self.lib,
            })
        } else {
            Err(AmdSmiError::Status(status))
        }
    }
}

impl Drop for AmdSmi {
    /// Uses `amdsmi_shut_down` for cleanup.
    fn drop(&mut self) {
        // SAFETY: amdsmi_shut_down has signature fn() -> amdsmi_status_t.
        // It is a cleanup function that takes no arguments. We ignore the return value.
        // It is safe to call during cleanup as it only releases library resources.
        let _ = unsafe { self.lib.amdsmi_shut_down.as_ref().map(|f| f()) };
    }
}

/// A handle to a device - currently only supports AMD GPUs.
pub struct Entity<'a> {
    handle: amdsmi_processor_handle,
    lib: &'a libamd_smi,
}

impl<'a> Entity<'a> {
    /// Get a human-readable string for an AMD SMI status code using `amdsmi_status_code_to_string`
    fn status_code_to_string(&self, status: amdsmi_status_t) -> String {
        let mut msg_ptr: *const std::os::raw::c_char = std::ptr::null();
        // SAFETY: amdsmi_status_code_to_string has signature
        // fn(status: amdsmi_status_t, status_string: *mut *const c_char) -> amdsmi_status_t.
        // We pass a valid status code and a pointer to a null pointer.
        // The function will write a string pointer to msg_ptr on success.
        let result = unsafe {
            self.lib
                .amdsmi_status_code_to_string
                .as_ref()
                .map(|f| f(status, &mut msg_ptr))
                .unwrap_or(amdsmi_status_t::AMDSMI_STATUS_FAIL_LOAD_SYMBOL)
        };

        if result == amdsmi_status_t::AMDSMI_STATUS_SUCCESS && !msg_ptr.is_null() {
            // SAFETY: msg_ptr is a valid C string pointer written by amdsmi_status_code_to_string
            // when the call returns AMDSMI_STATUS_SUCCESS. We check both the return status and
            // that msg_ptr is not null before dereferencing.
            unsafe { CStr::from_ptr(msg_ptr).to_string_lossy().into_owned() }
        } else {
            format!("{:?}", status)
        }
    }

    /// Get the list of possible system clock speeds of device for a specified clock type. It is
    /// not supported on virtual machine guest
    pub fn get_clk_freq(&self) -> Result<Frequencies, AmdSmiError> {
        let clk_type = amdsmi_clk_type_t::AMDSMI_CLK_TYPE_GFX;
        let mut freqs = MaybeUninit::<amdsmi_frequencies_t>::uninit();
        // SAFETY: amdsmi_get_clk_freq has signature
        // fn(handle, clk_type, freqs: *mut amdsmi_frequencies_t) -> amdsmi_status_t.
        // On success, the function writes the frequencies structure to the pointer we provide.
        // We check the return status before using the value, so assume_init is valid.
        let (status, freqs_init) = unsafe {
            let status = self
                .lib
                .amdsmi_get_clk_freq
                .as_ref()
                .expect("amdsmi_get_clk_freq is not available")(
                self.handle,
                clk_type,
                freqs.as_mut_ptr(),
            );
            (status, freqs.assume_init())
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(Frequencies::from(freqs_init))
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Returns the current usage of the GPU engines (GFX, MM and MEM). Each usage is reported as a
    /// percentage from 0-100%.
    pub fn get_gpu_activity(&self) -> Result<EngineUsage, AmdSmiError> {
        let mut usage = MaybeUninit::<amdsmi_engine_usage_t>::uninit();
        // SAFETY: amdsmi_get_gpu_activity has signature
        // fn(handle, usage: *mut amdsmi_engine_usage_t) -> amdsmi_status_t.
        // On success, the function writes the usage structure to the pointer we provide.
        // We check the return status before using the value, so assume_init is valid.
        let (status, usage) = unsafe {
            let status = self
                .lib
                .amdsmi_get_gpu_activity
                .as_ref()
                .expect("amdsmi_get_gpu_activity is not available")(
                self.handle,
                usage.as_mut_ptr(),
            );
            (status, usage.assume_init())
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(EngineUsage::from(usage))
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Get the performance level of the device. It is not supported on virtual machine guest
    pub fn get_gpu_perf_level(&self) -> Result<PerfLevel, AmdSmiError> {
        let mut perf_level = MaybeUninit::<PerfLevel>::uninit();
        // SAFETY: amdsmi_get_gpu_perf_level has signature
        // fn(handle, perf_level: *mut amdsmi_dev_perf_level_t) -> amdsmi_status_t.
        // On success, the function writes the perf_level structure to the pointer we provide.
        // We check the return status before using the value, so assume_init is valid.
        let (status, perf_level) = unsafe {
            let status = self
                .lib
                .amdsmi_get_gpu_perf_level
                .as_ref()
                .expect("amdsmi_get_gpu_perf_level is not available")(
                self.handle,
                perf_level.as_mut_ptr(),
            );
            (status, perf_level.assume_init())
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(perf_level)
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Returns the power caps as currently configured in the system.
    pub fn get_power_cap_info(&self, sensor_id: u32) -> Result<PowerCapInfo, AmdSmiError> {
        let mut power_cap_info = MaybeUninit::<PowerCapInfo>::uninit();
        // SAFETY: amdsmi_get_power_cap_info has signature
        // fn(handle, sensor_id: u32, power_cap_info: *mut PowerCapInfo) -> amdsmi_status_t.
        // On success, the function writes the power_cap_info structure to the pointer we provide.
        // We check the return status before using the value, so assume_init is valid.
        let (status, power_cap_info) = unsafe {
            let status = self
                .lib
                .amdsmi_get_power_cap_info
                .as_ref()
                .expect("amdsmi_get_power_cap_info is not available")(
                self.handle,
                sensor_id,
                power_cap_info.as_mut_ptr(),
            );
            (status, power_cap_info.assume_init())
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(power_cap_info)
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Returns the current power and voltage of the GPU.
    pub fn get_power_info(&self) -> Result<PowerConsumption, AmdSmiError> {
        let mut power_info = MaybeUninit::<amdsmi_power_info_t>::uninit();
        // SAFETY: amdsmi_get_power_info has signature
        // fn(handle, power_info: *mut amdsmi_power_info_t) -> amdsmi_status_t.
        // On success, the function writes the power_info structure to the pointer we provide.
        // We check the return status before using the value, so assume_init is valid.
        let (status, power_info) = unsafe {
            let status = self
                .lib
                .amdsmi_get_power_info
                .as_ref()
                .expect("amdsmi_get_power_info is not available")(
                self.handle,
                power_info.as_mut_ptr(),
            );
            (status, power_info.assume_init())
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(PowerConsumption::from(power_info))
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Query the supported power cap sensors and their types for a device.
    pub fn get_supported_power_cap(&self) -> Result<PowerCapSensors, AmdSmiError> {
        let mut sensor_count: u32 = 0;
        let mut sensor_inds = [0u32; 16];
        let mut sensor_types = [amdsmi_power_cap_type_t::AMDSMI_POWER_CAP_TYPE_PPT0; 16];
        // SAFETY: amdsmi_get_supported_power_cap has signature
        // fn(handle, sensor_count: *mut u32, sensor_inds: *mut u32, sensor_types:
        // *mut amdsmi_power_cap_type_t) -> amdsmi_status_t.
        // We pass a valid handle and pointers to our allocated arrays.
        // The function will write the sensor count and populate the arrays on success.
        let status = unsafe {
            self.lib
                .amdsmi_get_supported_power_cap
                .as_ref()
                .expect("amdsmi_get_supported_power_cap is not available")(
                self.handle,
                &mut sensor_count,
                sensor_inds.as_mut_ptr(),
                sensor_types.as_mut_ptr(),
            )
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(PowerCapSensors {
                sensor_indices: sensor_inds[..sensor_count as usize].to_vec(),
                sensor_types: sensor_types[..sensor_count as usize].to_vec(),
            })
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Returns is power management enabled
    pub fn is_gpu_power_management_enabled(&self) -> Result<bool, AmdSmiError> {
        let mut enabled = MaybeUninit::<bool>::uninit();
        // SAFETY: amdsmi_is_gpu_power_management_enabled has signature
        // fn(handle, enabled: *mut bool) -> amdsmi_status_t.
        // On success, the function writes the bool value to the pointer we provide.
        // We check the return status before using the value, so assume_init is valid.
        let (status, enabled) = unsafe {
            let status = self
                .lib
                .amdsmi_is_gpu_power_management_enabled
                .as_ref()
                .expect("amdsmi_is_gpu_power_management_enabled is not available")(
                self.handle,
                enabled.as_mut_ptr(),
            );
            (status, enabled.assume_init())
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(enabled)
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Control the set of allowed frequencies that can be used for the specified clock. It is not
    /// supported on virtual machine guest
    pub fn set_clk_freq(&self, bitmask: u64) -> Result<(), AmdSmiError> {
        let clk_type = amdsmi_clk_type_t::AMDSMI_CLK_TYPE_GFX;
        // SAFETY: amdsmi_set_clk_freq has signature
        // fn(handle, clk_type, bitmask: u64) -> amdsmi_status_t.
        // We pass a valid handle, AMDSMI_CLK_TYPE_GFX, and a bitmask value. There is no pointer
        // dereferencing, only scalar arguments.
        let status = unsafe {
            self.lib
                .amdsmi_set_clk_freq
                .as_ref()
                .expect("amdsmi_set_clk_freq is not available")(
                self.handle, clk_type, bitmask
            )
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// This function sets the clock sets the clock min/max level
    pub fn set_gpu_clk_limit(&self, freq: u64, max: bool) -> Result<(), AmdSmiError> {
        let clk_type = amdsmi_clk_type_t::AMDSMI_CLK_TYPE_GFX;
        let limit_type = if max {
            amdsmi_clk_limit_type_t::CLK_LIMIT_MAX
        } else {
            amdsmi_clk_limit_type_t::CLK_LIMIT_MIN
        };
        // SAFETY: amdsmi_set_gpu_clk_limit has signature
        // fn(handle, clk_type, limit_type, freq: u64) -> amdsmi_status_t.
        // We pass a valid handle, AMDSMI_CLK_TYPE_GFX, a valid limit_type (CLK_LIMIT_MAX or MIN),
        // and a frequency value. There is no pointer dereferencing, only scalar arguments.
        let status = unsafe {
            self.lib
                .amdsmi_set_gpu_clk_limit
                .as_ref()
                .expect("amdsmi_set_gpu_clk_limit is not available")(
                self.handle,
                clk_type,
                limit_type,
                freq,
            )
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Enter performance determinism mode with provided processor handle. It is not supported on
    /// virtual machine guest
    pub fn set_gpu_perf_determinism_mode(&self, freq: u64) -> Result<(), AmdSmiError> {
        // SAFETY: amdsmi_set_gpu_perf_determinism_mode has signature
        // fn(handle, freq: u64) -> amdsmi_status_t.
        // We pass a valid handle and a frequency value. There is no pointer dereferencing, only
        // scalar arguments.
        let status = unsafe {
            self.lib
                .amdsmi_set_gpu_perf_determinism_mode
                .as_ref()
                .expect("amdsmi_set_gpu_perf_determinism_mode is not available")(
                self.handle, freq
            )
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Set the PowerPlay performance level associated with the device with provided processor
    /// handle with the provided value. It is not supported on virtual machine guest
    pub fn set_gpu_perf_level(&self, level: PerfLevel) -> Result<(), AmdSmiError> {
        // SAFETY: amdsmi_set_gpu_perf_level has signature
        // fn(handle, perf_level: amdsmi_dev_perf_level_t) -> amdsmi_status_t.
        // We pass a valid handle and a valid performance level enum value. There is no pointer
        // dereferencing, only scalar arguments.
        let status = unsafe {
            self.lib
                .amdsmi_set_gpu_perf_level
                .as_ref()
                .expect("amdsmi_set_gpu_perf_level is not available")(self.handle, level)
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }

    /// Set the maximum gpu power cap value. It is not supported on virtual machine guest
    pub fn set_power_cap(&self, sensor_id: u32, cap: u64) -> Result<(), AmdSmiError> {
        // SAFETY: amdsmi_set_power_cap has signature
        // fn(handle, sensor_id: u32, cap: u64) -> amdsmi_status_t.
        // We pass a valid handle and valid scalar arguments. There is no pointer
        // dereferencing.
        let status = unsafe {
            self.lib
                .amdsmi_set_power_cap
                .as_ref()
                .expect("amdsmi_set_power_cap is not available")(
                self.handle, sensor_id, cap
            )
        };
        if status == amdsmi_status_t::AMDSMI_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(AmdSmiError::Message(self.status_code_to_string(status)))
        }
    }
}

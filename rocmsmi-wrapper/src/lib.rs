// Copyright 2026 Sustain AI
// Licensed under the Apache License, Version 2.0 (the "License");
// SPDX-License-Identifier: Apache-2.0

//! Rust wrapper for ROCm SMI (System Management Interface)
//!
//! This crate provides a high-level, safe interface to the rocmsmi library for managing ROCm devices.

pub mod error;
pub mod types;

use rocmsmi_wrapper_sys::{RSMI_POWER_TYPE, librocm_smi, rsmi_status_t};

pub use error::RocmSmiError;
pub use types::{Bdf, PowerInfo, PowerType};

/// Main ROCm SMI interface
///
/// This struct manages the rocmsmi library and provides methods to interact with ROCm GPU devices.
pub struct RocmSmi {
    lib: librocm_smi,
}

// TODO: implement drop.

impl RocmSmi {
    /// Initialize the ROCm SMI library by loading `librocm_smi.so`
    pub fn init() -> Result<Self, RocmSmiError> {
        // SAFETY: librocm_smi::new loads the system library "librocm_smi64.so" at runtime.
        // This is safe because we're loading a standard system library with a hardcoded name.
        let lib = unsafe { librocm_smi::new("librocm_smi64.so")? };
        // Initialize the ROCm SMI library with default flags...
        // SAFETY: rsmi_init has signature fn(flags: u64) -> rsmi_status_t.
        // We pass 0 for default flags. This is a standard initialization function.
        let status = unsafe { lib.rsmi_init(0) };
        if status != rsmi_status_t::RSMI_STATUS_SUCCESS {
            return Err(RocmSmiError::Status(status));
        }
        Ok(RocmSmi { lib })
    }

    /// Get percentage of time device is busy doing any processing
    pub fn dev_busy_percent_get(&self, device_index: u32) -> Result<u32, RocmSmiError> {
        let mut busy: u32 = 0;
        // SAFETY: rsmi_dev_busy_percent_get has signature
        // fn(dv_ind: u32, busy_percent: *mut u32) -> rsmi_status_t.
        // We pass a valid device index and a pointer to an uninitialized variable.
        // The function will write to this pointer on success.
        let status = unsafe {
            self.lib
                .rsmi_dev_busy_percent_get
                .as_ref()
                .expect("rsmi_dev_busy_percent_get not loaded")(device_index, &mut busy)
        };
        if status == rsmi_status_t::RSMI_STATUS_SUCCESS {
            Ok(busy)
        } else {
            Err(RocmSmiError::Status(status))
        }
    }

    /// Get the PCI BDF ID for a device index
    pub fn dev_pci_id_get(&self, device_index: u32) -> Result<u64, RocmSmiError> {
        let mut bdfid: u64 = 0;
        // SAFETY: rsmi_dev_pci_id_get has signature
        // fn(dv_ind: u32, bdfid: *mut u64) -> rsmi_status_t.
        let status = unsafe {
            self.lib
                .rsmi_dev_pci_id_get
                .as_ref()
                .expect("rsmi_dev_pci_id_get not loaded")(device_index, &mut bdfid)
        };
        if status == rsmi_status_t::RSMI_STATUS_SUCCESS {
            Ok(bdfid)
        } else {
            Err(RocmSmiError::Status(status))
        }
    }

    /// Get the average power consumption of the device with provided device index
    pub fn dev_power_get(&self, device_index: u32) -> Result<PowerInfo, RocmSmiError> {
        let mut power: u64 = 0;
        let mut power_type: RSMI_POWER_TYPE = RSMI_POWER_TYPE::RSMI_INVALID_POWER;
        // SAFETY: rsmi_dev_power_get has signature
        // fn(dv_ind: u32, power: *mut u64, type_: *mut RSMI_POWER_TYPE) -> rsmi_status_t.
        // We pass a valid device index and pointers to uninitialized variables.
        // The function will write to these pointers on success.
        let status = unsafe {
            self.lib
                .rsmi_dev_power_get
                .as_ref()
                .expect("rsmi_dev_power_get not loaded")(
                device_index, &mut power, &mut power_type
            )
        };
        if status == rsmi_status_t::RSMI_STATUS_SUCCESS {
            Ok(PowerInfo {
                power,
                power_type: PowerType::from(power_type),
            })
        } else {
            Err(RocmSmiError::Status(status))
        }
    }

    /// Get the number of monitor devices
    pub fn num_monitor_devices(&self) -> Result<u32, RocmSmiError> {
        let mut num_devices: u32 = 0;
        // SAFETY: rsmi_num_monitor_devices has signature
        // fn(num_devices: *mut u32) -> rsmi_status_t.
        // We pass a pointer to an uninitialized variable.
        // The function will write the number of devices to this pointer on success.
        let status = unsafe {
            self.lib
                .rsmi_num_monitor_devices
                .as_ref()
                .expect("rsmi_num_monitor_devices not loaded")(&mut num_devices)
        };
        if status == rsmi_status_t::RSMI_STATUS_SUCCESS {
            Ok(num_devices)
        } else {
            Err(RocmSmiError::Status(status))
        }
    }

    /// Get the device index for a device with the specified BDF
    pub fn get_device_index_by_bdf(&self, bdf_str: &str) -> Result<u32, RocmSmiError> {
        let target_bdf: Bdf = bdf_str.parse().map_err(RocmSmiError::Message)?;
        let num_devices = self.num_monitor_devices()?;

        for dv_ind in 0..num_devices {
            let mut bdfid: u64 = 0;
            // SAFETY: rsmi_dev_pci_id_get has signature
            // fn(dv_ind: u32, bdfid: *mut u64) -> rsmi_status_t.
            // We pass a valid device index and a pointer to an uninitialized variable.
            let status = unsafe {
                self.lib
                    .rsmi_dev_pci_id_get
                    .as_ref()
                    .expect("rsmi_dev_pci_id_get not loaded")(dv_ind, &mut bdfid)
            };
            if status != rsmi_status_t::RSMI_STATUS_SUCCESS {
                continue;
            }
            if bdfid == target_bdf.as_uint {
                return Ok(dv_ind);
            }
        }
        Err(RocmSmiError::Message(format!(
            "No device found with BDF: {}",
            bdf_str
        )))
    }
}

impl Drop for RocmSmi {
    /// Uses `rsmi_shut_down` for cleanup.
    fn drop(&mut self) {
        // SAFETY: rsmi_shut_down has signature fn() -> rsmi_status_t.
        // It is a cleanup function that takes no arguments. We ignore the return value.
        // It is safe to call during cleanup as it only releases library resources.
        let _ = unsafe { self.lib.rsmi_shut_down.as_ref().map(|f| f()) };
    }
}

// Copyright 2026 Sustain AI
// Licensed under the Apache License, Version 2.0 (the "License");
// SPDX-License-Identifier: Apache-2.0

pub use amdsmi_wrapper_sys::amdsmi_dev_perf_level_t as PerfLevel;
pub use amdsmi_wrapper_sys::amdsmi_power_cap_info_t as PowerCapInfo;
pub use amdsmi_wrapper_sys::amdsmi_power_cap_type_t as PowerCapType;
use amdsmi_wrapper_sys::{
    amdsmi_bdf_t, amdsmi_engine_usage_t, amdsmi_frequencies_t, amdsmi_power_info_t,
};
use std::str::FromStr;

/// Parameters about `amdsmi_bdf_t`.
#[derive(Debug, Default, Clone)]
pub struct Bdf {
    /// The BDF value as a packed uint.
    pub as_uint: u64,
}

impl FromStr for Bdf {
    type Err = String;

    fn from_str(bdf_str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = bdf_str.split([':', '.']).collect();
        if parts.len() != 4 {
            return Err(format!(
                "Invalid BDF format: '{}'. Expected format: DDDD:BB:DD.F",
                bdf_str
            ));
        }
        let domain = u64::from_str_radix(parts[0], 16)
            .map_err(|_| format!("Invalid domain: '{}'", parts[0]))?;
        let bus = u64::from_str_radix(parts[1], 16)
            .map_err(|_| format!("Invalid bus: '{}'", parts[1]))?;
        let device = u64::from_str_radix(parts[2], 16)
            .map_err(|_| format!("Invalid device: '{}'", parts[2]))?;
        let function = u64::from_str_radix(parts[3], 16)
            .map_err(|_| format!("Invalid function: '{}'", parts[3]))?;

        if bus > 0xFF {
            return Err(format!("Bus {} out of range (0-255)", bus));
        }
        if device > 0x1F {
            return Err(format!("Device {} out of range (0-31)", device));
        }
        if function > 0x7 {
            return Err(format!("Function {} out of range (0-7)", function));
        }

        Ok(Self {
            as_uint: (domain << 48) | (bus << 8) | (device << 3) | function,
        })
    }
}

impl From<Bdf> for amdsmi_bdf_t {
    fn from(bdf: Bdf) -> Self {
        Self {
            as_uint: bdf.as_uint,
        }
    }
}

/// Parameters about `amdsmi_frequencies_t`.
#[derive(Debug, Default, Clone)]
pub struct Frequencies {
    /// List of supported frequencies in MHz.
    pub frequencies: Vec<u64>,
    // omitting other fields
}

impl From<amdsmi_frequencies_t> for Frequencies {
    fn from(value: amdsmi_frequencies_t) -> Self {
        Self {
            frequencies: value.frequency[..value.num_supported as usize].to_vec(),
        }
    }
}

/// Parameters about the engine activity usage: `amdsmi_engine_usage_t`.
#[derive(Debug, Default, Clone)]
pub struct EngineUsage {
    /// Main graphic core of AMD GPU, in percentage.
    pub gfx_activity: u32,
    // omitting other fields
}

impl From<amdsmi_engine_usage_t> for EngineUsage {
    fn from(info: amdsmi_engine_usage_t) -> Self {
        Self {
            gfx_activity: info.gfx_activity,
        }
    }
}

/// Parameters about power consumption: `amdsmi_power_info_t`.
#[derive(Debug, Default, Clone)]
pub struct PowerConsumption {
    /// Average socket power in W, Navi + Mi 200 and earlier Series cards.
    pub average_socket_power: u32,
    // omitting other fields
}

impl From<amdsmi_power_info_t> for PowerConsumption {
    fn from(info: amdsmi_power_info_t) -> Self {
        Self {
            average_socket_power: info.average_socket_power,
        }
    }
}

/// Information about supported power cap sensors
#[derive(Debug, Default, Clone)]
pub struct PowerCapSensors {
    /// List of sensor indices
    pub sensor_indices: Vec<u32>,
    /// List of sensor types (e.g., PPT0, PPT1)
    pub sensor_types: Vec<PowerCapType>,
}

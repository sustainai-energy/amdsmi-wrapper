// Copyright 2026 Sustain AI
// Licensed under the Apache License, Version 2.0 (the "License");
// SPDX-License-Identifier: Apache-2.0

use rocmsmi_wrapper_sys::RSMI_POWER_TYPE;
use std::str::FromStr;

/// A BDF (Bus:Device.Function) identifier for a PCI device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bdf {
    /// The BDF value as a packed uint64_t matching the ROCm SMI format.
    pub as_uint: u64,
}

impl Bdf {
    /// Parse a BDF string in format "DDDD:BB:DD.F"
    pub fn parse(bdf_str: &str) -> Result<Self, String> {
        bdf_str.parse()
    }
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

        // ROCm SMI BDFID format: ((DOMAIN & 0xFFFFFFFF) << 32) | ((BUS & 0xFF) << 8) | ((DEVICE & 0x1F) << 3) | (FUNCTION & 0x7)
        Ok(Self {
            as_uint: ((domain & 0xFFFFFFFF) << 32)
                | ((bus & 0xFF) << 8)
                | ((device & 0x1F) << 3)
                | (function & 0x7),
        })
    }
}

/// Power type for ROCm SMI power readings
#[derive(Debug, Clone, Copy)]
pub enum PowerType {
    /// Average power
    Average,
    /// Current/instantaneous power
    Current,
    /// Invalid power reading
    Invalid,
}

impl From<RSMI_POWER_TYPE> for PowerType {
    fn from(value: RSMI_POWER_TYPE) -> Self {
        match value {
            RSMI_POWER_TYPE::RSMI_AVERAGE_POWER => PowerType::Average,
            RSMI_POWER_TYPE::RSMI_CURRENT_POWER => PowerType::Current,
            RSMI_POWER_TYPE::RSMI_INVALID_POWER => PowerType::Invalid,
        }
    }
}

impl From<PowerType> for RSMI_POWER_TYPE {
    fn from(value: PowerType) -> Self {
        match value {
            PowerType::Average => RSMI_POWER_TYPE::RSMI_AVERAGE_POWER,
            PowerType::Current => RSMI_POWER_TYPE::RSMI_CURRENT_POWER,
            PowerType::Invalid => RSMI_POWER_TYPE::RSMI_INVALID_POWER,
        }
    }
}

/// Power reading from a device
#[derive(Debug, Clone, Copy)]
pub struct PowerInfo {
    /// Power value in milliwatts
    pub power: u64,
    /// Type of power reading
    pub power_type: PowerType,
}

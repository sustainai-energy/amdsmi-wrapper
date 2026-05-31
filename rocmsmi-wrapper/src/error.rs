// Copyright 2026 Sustain AI
// Licensed under the Apache License, Version 2.0 (the "License");
// SPDX-License-Identifier: Apache-2.0

use rocmsmi_wrapper_sys::rsmi_status_t;
use thiserror::Error;

/// Error types for ROCm SMI wrapper
#[derive(Debug, Error)]
pub enum RocmSmiError {
    #[error("ROCm SMI error: {0:?}")]
    Status(rsmi_status_t),
    #[error("Library load error: {0}")]
    Load(#[from] libloading::Error),
    #[error("Message: {0}")]
    Message(String),
}

/// Fallback conversion from rsmi_status_t error code to Error
impl From<rsmi_status_t> for RocmSmiError {
    fn from(status: rsmi_status_t) -> Self {
        RocmSmiError::Status(status)
    }
}

/// Support for basic String conversion to Error
impl From<String> for RocmSmiError {
    fn from(s: String) -> Self {
        RocmSmiError::Message(s)
    }
}

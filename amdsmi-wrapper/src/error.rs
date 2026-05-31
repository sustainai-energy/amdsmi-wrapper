// Copyright 2026 Sustain AI
// Licensed under the Apache License, Version 2.0 (the "License");
// SPDX-License-Identifier: Apache-2.0

use amdsmi_wrapper_sys::amdsmi_status_t;
use thiserror::Error;

/// Error types
#[derive(Debug, Error)]
pub enum AmdSmiError {
    #[error("AMD SMI error: {0:?}")]
    Status(amdsmi_status_t),
    #[error("Library load error: {0}")]
    Load(#[from] libloading::Error),
    #[error("Parse error: {0}")]
    Message(String),
}

/// Fallback conversion from amdsmi_status_t error code to Error
impl From<amdsmi_status_t> for AmdSmiError {
    fn from(status: amdsmi_status_t) -> Self {
        AmdSmiError::Status(status)
    }
}

/// Support for basic String conversion to Error
impl From<String> for AmdSmiError {
    fn from(s: String) -> Self {
        AmdSmiError::Message(s)
    }
}

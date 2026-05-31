
# amdsmi-wrapper

Wraps [AMD System Management Interface (AMD SMI) library](https://github.com/ROCm/rocm-systems/tree/develop/projects/amdsmi).

## Build

By default, only the amdsmi wrappers are compiled. To build all crates including rocmsmi-wrapper:

```bash
cargo build --workspace
```

To use a custom AMD SMI header location for amdsmi-wrapper-sys, set the `AMDSMI_INCLUDE_PATH` environment variable:

```bash
AMDSMI_INCLUDE_PATH=/opt/rocm-7.2.3/include cargo build
```

To build against a specific ROCm version, set the `ROCM_INCLUDE_PATH` environment variable to point to your ROCm include directory (default: `/opt/rocm-7.2.3/include`):

```bash
ROCM_INCLUDE_PATH=/opt/rocm-7.2.3/include cargo build --workspace
```

# License

* Copyright (c) 2026 Sustain AI
* Distributed under the Apache License, Version 2.0.

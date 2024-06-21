# Error Tree

## Overview
`error_tree` is a Rust procedural macro crate designed to simplify error handling in Rust applications. It allows you to define complex error hierarchies in a clean and structured way, making your error handling logic both more robust and easier to maintain.

## Features
- Define nested error enums with ease.
- Automatically generate `From` implementations for error conversions.
- Simplify complex error handling in Rust.

## Installation
Add `error_tree` to your `Cargo.toml` file under `[dependencies]`:
```toml
[dependencies]
error_tree = "0.1.0"
```

## Usage
Here's a basic example of how to use `error_tree`:
```rust
use error_tree::error_tree;

error_tree!{

    // Enumerate possible errors for capturing audio.
    pub enum PassiveAudioCaptureError {
        FormatError,
        DeviceError(DeviceError),
        IOError(IOError),
        WavError(WavError),
        HostError(HostError),
        StreamOrChannelError(StreamOrChannelError),
    }

    pub enum MonitorAllInputsError { 
        DeviceError(DeviceError),
    }

    pub enum StreamOrChannelError { 
        StreamError(StreamError),
        ChannelError(ChannelError),
    }

    pub enum IOError { 
        Basic(std::io::Error),
    }

    pub enum ChannelError { 
        ChannelRecvError(mpsc::RecvError),
    }

    pub enum StreamError { 
        StreamError(cpal::StreamError),
        PauseStreamError(cpal::PauseStreamError),
        BuildStreamError(cpal::BuildStreamError),
        PlayStreamError(cpal::PlayStreamError),
        SupportedStreamConfigsError(cpal::SupportedStreamConfigsError),
        DefaultStreamConfigError(cpal::DefaultStreamConfigError),
    }

    pub enum DeviceError { 
        DeviceNotAvailable {
            device_name: String,
        },

        Basic(cpal::DevicesError),
        NameError(cpal::DeviceNameError),
    }

    pub enum WavError { 
        Hound(hound::Error),
    }

    pub enum HostError { 
        HostUnavailable(cpal::HostUnavailable),
    }
}

```

This will automatically generate the necessary `From` implementations for error conversions within your defined hierarchy.

## Contributing
Contributions to `error_tree` are welcome! Please read our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.

## License
This crate is distributed under the terms of MIT License.

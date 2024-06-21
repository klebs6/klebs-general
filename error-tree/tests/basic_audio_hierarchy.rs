use error_tree::error_tree;
use std::sync::mpsc;

#[test]
fn test_error_tree() {

    error_tree!{

        // Enumerate possible errors for capturing audio.
        pub enum PassiveAudioCaptureError {
            FormatError,
            DeviceError(DeviceError),
            IOError(IOError),
            WavError(WavError),
            HostError(HostError),
            StreamError(StreamError),
            ChannelError(ChannelError),
            ListCpal(ListCpalHostsError),
        }

        pub enum MonitorAllInputsError { 
            DevicesError(DeviceError),
        }

        pub enum ListCpalHostsError { 
            Default,
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

    let e1 = cpal::StreamError::DeviceNotAvailable;

    let e2: cpal::DeviceNameError 
        = cpal::BackendSpecificError { description: "error".into() }.into();

    // Perform your assertions here
    // For example, to test if `DeviceError` can be converted to `PassiveAudioCaptureError`
    let f1: PassiveAudioCaptureError = e1.into();

    let f2: PassiveAudioCaptureError = e2.into();

    match f1 {
        PassiveAudioCaptureError::StreamError(e) => {
            println!("e1 {:#?}",e);
        },
        _ => {}
    }

    match f2 {
        PassiveAudioCaptureError::DeviceError(e) => {
            println!("e2 {:#?}",e);
        },
        _ => {}
    }
}

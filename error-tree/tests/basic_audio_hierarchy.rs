use error_tree::error_tree;
use std::sync::mpsc;

#[test]
fn test_error_tree() {

    #[derive(Debug)]
    pub enum CpalStreamError {
        DeviceNotAvailable,
    }
    #[derive(Debug)]
    pub struct CpalBackendSpecificError {
        description: String,
    }

    impl From<CpalBackendSpecificError> for CpalDeviceNameError {

        fn from(x: CpalBackendSpecificError) -> Self {
            CpalDeviceNameError
        }
    }

    #[derive(Debug)] pub struct CpalPauseStreamError;
    #[derive(Debug)] pub struct CpalBuildStreamError;
    #[derive(Debug)] pub struct CpalPlayStreamError;
    #[derive(Debug)] pub struct CpalSupportedStreamConfigsError;
    #[derive(Debug)] pub struct CpalDefaultStreamConfigError;
    #[derive(Debug)] pub struct CpalDevicesError;
    #[derive(Debug)] pub struct CpalDeviceNameError;
    #[derive(Debug)] pub struct CpalHostUnavailable;
    #[derive(Debug)] pub struct HoundError;

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
            StreamError(CpalStreamError),
            PauseStreamError(CpalPauseStreamError),
            BuildStreamError(CpalBuildStreamError),
            PlayStreamError(CpalPlayStreamError),
            SupportedStreamConfigsError(CpalSupportedStreamConfigsError),
            DefaultStreamConfigError(CpalDefaultStreamConfigError),
        }

        pub enum DeviceError { 
            DeviceNotAvailable {
                device_name: String,
            },

            Basic(CpalDevicesError),
            NameError(CpalDeviceNameError),
        }

        pub enum WavError { 
            Hound(HoundError),
        }

        pub enum HostError { 
            HostUnavailable(CpalHostUnavailable),
        }
    }

    let e1 = CpalStreamError::DeviceNotAvailable;

    let e2: CpalDeviceNameError 
        = CpalBackendSpecificError { description: "error".into() }.into();

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

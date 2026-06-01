crate::ix!();

#[derive(Clone,Debug)]
pub struct ErrorTree {
    pub(crate) enums: Vec<ErrorEnum>,
}

impl TryFrom<proc_macro::TokenStream> for ErrorTree {

    type Error = syn::Error;

    fn try_from(input: proc_macro::TokenStream) -> Result<Self, syn::Error> {
        let input = proc_macro2::TokenStream::from(input);
        parse2(input)
    }
}

impl Parse for ErrorTree {

    fn parse(input: ParseStream) -> SynResult<Self> {

        let mut enums = Vec::new();

        while !input.is_empty() {
            let e = input.parse::<ErrorEnum>()?;
            enums.push(e);
        }

        Ok(enums.into())
    }
}

impl ToTokens for ErrorTree {

    fn to_tokens(&self, tokens: &mut TokenStream2) {

        self.enums.iter().for_each(|x| x.to_tokens(tokens));

        let from_impls: Vec<FromImplGenerationConfig> = self.into();

        from_impls.iter().for_each(|x| x.to_tokens(tokens));
    }
}

impl From<Vec<ErrorEnum>> for ErrorTree {

    fn from(enums: Vec<ErrorEnum>) -> Self {
        Self { enums }
    }
}

impl Validate for ErrorTree {

    fn validate(&self) -> bool {

        // Check for duplicate enum names
        let mut enum_names = HashSet::new();

        for error_enum in &self.enums {

            if !enum_names.insert(&error_enum.ident) {
                return false; // Duplicate enum name found
            }

            // Additional checks for each enum
            if !error_enum.validate() {
                return false;
            }
        }

        // Other global checks can be added here

        true // Passes all checks
    }
}

#[test] fn test_parse() {

    let input_str = r#"
        pub enum FirstError {
            FormatError,
            IOError(std::io::Error),
            DeviceNotAvailable { device_name: String }
        }
        pub enum SecondError {
            AnotherError
        }
    "#;

    let parse_result: Result<ErrorTree, syn::Error> = syn::parse_str(input_str);

    match parse_result {
        Ok(parsed_tree) => println!("Parsed successfully: {:#?}", parsed_tree),
        Err(e) => panic!("Failed to parse: {}", e),
    }
}

#[test] fn test_parse_advanced() {

    let input_str = r#"

        // Enumerate possible errors for capturing audio.
        pub enum PassiveAudioCaptureError {
            FormatError,
            DeviceError(DeviceError),
            IOError(IOError),
            WavError(WavError),
            HostError(HostError),
            StreamError(StreamError),
            ChannelError(ChannelError),
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
    "#;

    let parse_result: Result<ErrorTree, syn::Error> = syn::parse_str(input_str);

    match parse_result {
        Ok(parsed_tree) => println!("Parsed successfully: {:#?}", parsed_tree),
        Err(e) => panic!("Failed to parse: {}", e),
    }
}

#[test]
fn struct_variant_field_does_not_generate_from_impl_by_default() {

    let input_str = r#"
        pub enum RootError {
            Structured {
                payload: PayloadError
            }
        }

        pub enum PayloadError {
            Leaf
        }
    "#;

    let parse_result: Result<ErrorTree, syn::Error> = syn::parse_str(input_str);

    match parse_result {
        Ok(parsed_tree) => {
            let output = parsed_tree.into_token_stream().to_string();

            assert!(!output.contains("From < PayloadError > for RootError"), "{output}");
        }
        Err(e) => panic!("Failed to parse: {}", e),
    }
}

#[test]
fn generated_default_display_for_struct_variant_does_not_bind_fields() {

    let input_str = r#"
        pub enum MyError {
            DeviceNotAvailable {
                device_name: String
            }
        }
    "#;

    let parse_result: Result<ErrorTree, syn::Error> = syn::parse_str(input_str);

    match parse_result {
        Ok(parsed_tree) => {
            let output = parsed_tree.into_token_stream().to_string();

            assert!(output.contains("DeviceNotAvailable { .. }"), "{output}");
            assert!(!output.contains("DeviceNotAvailable { device_name }"), "{output}");
        }
        Err(e) => panic!("Failed to parse: {}", e),
    }
}

#[test]
fn generated_default_display_for_wrapped_variant_does_not_debug_payload() {

    let input_str = r#"
        pub enum MyError {
            IOError(std::io::Error)
        }
    "#;

    let parse_result: Result<ErrorTree, syn::Error> = syn::parse_str(input_str);

    match parse_result {
        Ok(parsed_tree) => {
            let output = parsed_tree.into_token_stream().to_string();

            assert!(output.contains("IOError (_)"), "{output}");
            assert!(!output.contains("inner"), "{output}");
            assert!(!output.contains(": ?"), "{output}");
        }
        Err(e) => panic!("Failed to parse: {}", e),
    }
}

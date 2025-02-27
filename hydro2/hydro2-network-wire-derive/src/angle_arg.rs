// ---------------- [ File: src/angle_arg.rs ]
crate::ix!();

/// We describe each angle-bracketed argument as either
/// something that reuses a generic from the wire struct
/// or a freshly created param like `OpTy0`.
#[derive(Clone, Debug)]
pub enum AngleArg {
    /// We found an ident that matches an existing wire generic
    /// (e.g. user wrote `op="ConstantOp<Z>"` and wire also has `Z`).
    /// So we unify them with the same `Ident`.
    Reused(Ident),

    /// We minted a brand-new parameter (e.g. `OpTy0`)
    /// because the user’s angle arg did not match any existing wire param.
    Fresh(Ident),

    /// For any type/const that is not recognized as a wire param.
    Literal(syn::GenericArgument),
}

#[cfg(test)]
mod test_angle_arg {
    use super::*;
    use quote::ToTokens;
    use syn::Ident;
    use proc_macro2::Span;

    #[test]
    fn test_angle_arg_reused_creation() {
        let id = Ident::new("Z", Span::call_site());
        let arg = AngleArg::Reused(id.clone());
        match arg {
            AngleArg::Reused(ref ident) => {
                assert_eq!(ident.to_string(), "Z");
            }
            AngleArg::Fresh(_) => panic!("Expected Reused variant"),
            _ => {}
        }
    }

    #[test]
    fn test_angle_arg_fresh_creation() {
        let id = Ident::new("OpTy0", Span::call_site());
        let arg = AngleArg::Fresh(id.clone());
        match arg {
            AngleArg::Fresh(ref ident) => {
                assert_eq!(ident.to_string(), "OpTy0");
            }
            AngleArg::Reused(_) => panic!("Expected Fresh variant"),
            _ => {}
        }
    }

    #[test]
    fn test_angle_arg_clone() {
        let original = AngleArg::Fresh(Ident::new("CloneTest", Span::call_site()));
        let cloned = original.clone();
        match cloned {
            AngleArg::Fresh(id) => assert_eq!(id.to_string(), "CloneTest"),
            AngleArg::Reused(_) => panic!("Expected Fresh variant"),
            _ => {}
        }
    }

    #[test]
    fn test_angle_arg_debug() {
        let id = Ident::new("ABC", Span::call_site());
        let reused = AngleArg::Reused(id.clone());
        let fresh = AngleArg::Fresh(id);
        let reused_str = format!("{:?}", reused);
        let fresh_str = format!("{:?}", fresh);

        // We won't test the exact debug string, but we can check whether it has the variant name.
        assert!(reused_str.contains("Reused("), "Got: {}", reused_str);
        assert!(fresh_str.contains("Fresh("), "Got: {}", fresh_str);
    }
}

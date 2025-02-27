// ---------------- [ File: hydro2-network-wire-derive/src/build_single_operator_impl.rs ]
crate::ix!();

/// Builds bridging code (`impl Operator<WireEnum> for FooOp<...>`) for a single operator item.
/// 
/// - `network_wire_enum_ident`: the name of the wire enum (e.g. `MyWireIO`)
/// - `op_item`: the operator spec item (already finalized with minted/reused parameters)
/// - `sig_map`: a map from operator basename -> operator signature ident (if needed)
/// - `wire_generics`: generics of the wire type
pub fn build_single_operator_impl(
    network_wire_enum_ident: &syn::Ident,
    op_item:                 &OperatorSpecItem,
    sig_map:                 &std::collections::HashMap<String, syn::Ident>,
    wire_generics:           &syn::Generics,

) -> proc_macro2::TokenStream {

    info!("[build_single_operator_impl] START");
    info!("  network_wire_enum_ident = {}", network_wire_enum_ident);
    info!("  op_item = {:?}", op_item);
    info!("  wire_generics = {:?}", wire_generics);

    // (1) Extract the base operator name for lookup, e.g. "AddOp".
    let op_path = op_item.path().clone();
    let op_ident_str = match op_path.segments.last() {
        Some(seg) => seg.ident.to_string(),
        None => {
            return syn::Error::new(
                op_item.path().span(),
                "Operator path had no segments"
            ).to_compile_error();
        }
    };

    // (2) Find or fallback for the operator signature ident, e.g. "AddOpOperatorSignature".
    let operator_signature_ident = find_operator_signature_ident(sig_map, &op_ident_str, &op_path);
    info!("  operator_signature_ident = {}", operator_signature_ident);

    // (3) Merge wire + op generics purely in the AST.
    let merged_gens = unify_generics_ast(wire_generics, op_item.op_generics());

    // (4) Split those merged generics into impl/ty/wc so we can quote them in final output.
    let (impl_gen, ty_gen, wc_opt) = merged_gens.split_for_impl();
    info!(
        "  after split_for_impl => impl_gen = `{}`, ty_gen = `{}`, wc_opt = `{:?}`",
        impl_gen.to_token_stream(),
        ty_gen.to_token_stream(),
        wc_opt.as_ref().map(|wc| wc.to_token_stream())
    );

    // (5) The trait type is the user’s wire: e.g. MyWireIO<T>
    let (_wire_impl_gen, wire_ty_gen, _wire_wc) = wire_generics.split_for_impl();
    let operator_trait_type = quote::quote! {
        #network_wire_enum_ident #wire_ty_gen
    };

    let operator_trait_type_turbo = if quote!{ #wire_ty_gen }.to_string() == "".to_string() {
        quote::quote! {
            #network_wire_enum_ident
        }
    } else {
        quote::quote! {
            #network_wire_enum_ident :: #wire_ty_gen
        }
    };

    // (6) Build the mutated “IO path”: e.g. AddOp -> AddOpIO
    let (mutated_io_path, new_io_ident) = finalize_operator_io_path(op_path);
    info!(
        "  mutated_io_path = {}, new_io_ident = {}",
        mutated_io_path.to_token_stream(),
        new_io_ident
    );

    // (7) Rebuild the angle brackets from op_item.final_args() for the *base operator type* 
    // (FooOp<T> in the `impl ... for FooOp<T>`). Then also build the bridging `<...>` 
    // for the “IO path” references (FooOpIO<T>).
    let base_ident = match op_item.path().segments.last() {
        Some(seg) => seg.ident.clone(),
        None => syn::Ident::new("____ErrorNoSegment", op_item.path().span()),
    };

    let bridging_args = build_operator_type_args(op_item.final_args());
    let bridging_path = quote::quote! { #mutated_io_path #bridging_args };

    // (8) Generate final bridging code, using #bridging_path wherever we reference “FooOpIO<...>”.
    let expanded = quote::quote! {
        const _: () = {

            type Input0 #bridging_args = <#operator_signature_ident #bridging_args as hydro2_operator::OperatorSignature>::Input0;
            type Input1 #bridging_args = <#operator_signature_ident #bridging_args as hydro2_operator::OperatorSignature>::Input1;
            type Input2 #bridging_args = <#operator_signature_ident #bridging_args as hydro2_operator::OperatorSignature>::Input2;
            type Input3 #bridging_args = <#operator_signature_ident #bridging_args as hydro2_operator::OperatorSignature>::Input3;

            #[async_trait::async_trait]
            impl #impl_gen
                hydro2_operator::Operator<#operator_trait_type>
                    for #base_ident #bridging_args
                        #wc_opt
                    {
                        fn opcode(&self) -> std::sync::Arc<dyn hydro2_operator::OpCode> {
                            <Self as hydro2_operator::Operator<#bridging_path>>::opcode(self)
                        }

                        fn input_count(&self) -> usize {
                            <Self as hydro2_operator::Operator<#bridging_path>>::input_count(self)
                        }

                        fn output_count(&self) -> usize {
                            <Self as hydro2_operator::Operator<#bridging_path>>::output_count(self)
                        }

                        fn input_port_type_str(&self, port: usize) -> Option<&'static str> {
                            <Self as hydro2_operator::Operator<#bridging_path>>::input_port_type_str(self, port)
                        }

                        fn output_port_type_str(&self, port: usize) -> Option<&'static str> {
                            <Self as hydro2_operator::Operator<#bridging_path>>::output_port_type_str(self, port)
                        }

                        fn input_port_connection_required(&self, port: usize) -> bool {
                            <Self as hydro2_operator::Operator<#bridging_path>>::input_port_connection_required(self, port)
                        }

                        fn output_port_connection_required(&self, port: usize) -> bool {
                            <Self as hydro2_operator::Operator<#bridging_path>>::output_port_connection_required(self, port)
                        }

                        async fn execute(
                            &self,
                            input:  [Option<& #operator_trait_type>; 4],
                            output: &mut [Option<#operator_trait_type>; 4],
                        ) -> hydro2_operator::NetResult<()> {

                            use hydro2_operator::{
                                PortTryFrom0,
                                PortTryFrom1,
                                PortTryFrom2,
                                PortTryFrom3,
                                PortTryInto0,
                                PortTryInto1,
                                PortTryInto2,
                                PortTryInto3,
                                PortTryInto0Any,
                                PortTryInto1Any,
                                PortTryInto2Any,
                                PortTryInto3Any,
                            };

                            let mut adapted_in:  [Option<#bridging_path>; 4] = [None, None, None, None];
                            let mut adapted_out: [Option<#bridging_path>; 4] = [None, None, None, None];

                            //----------------------------------------------------[port-0]
                            if let Some(wire_val) = input[0] {
                                let i0 = wire_val.port_try_into_dynamic::<Input0 #bridging_args>()
                                    .expect(&format!("could not unpack wire val {:#?} into i0 of ty={}", wire_val, std::any::type_name::<Input0 #bridging_args>()));

                                let op_i0 = <#bridging_path as PortTryFrom0<Input0 #bridging_args>>::port_try_from0(i0)
                                    .expect("expected to be able to wire into port 0");

                                adapted_in[0] = Some(op_i0);
                            }

                            //----------------------------------------------------[port-1]
                            if let Some(wire_val) = input[1] {

                                let i1 = wire_val.port_try_into_dynamic::<Input1 #bridging_args>()
                                    .expect(&format!("could not unpack wire val {:#?} into i1 of ty={}", wire_val, std::any::type_name::<Input1 #bridging_args>()));

                                let op_i1 = <#bridging_path as PortTryFrom1<Input1 #bridging_args>>::port_try_from1(i1)
                                    .expect("expected to be able to wire into port 1");

                                adapted_in[1] = Some(op_i1);
                            }

                            //----------------------------------------------------[port-2]
                            if let Some(wire_val) = input[2] {

                                let i2 = wire_val.port_try_into_dynamic::<Input2 #bridging_args>()
                                    .expect(&format!("could not unpack wire val {:#?} into i2 of ty={}", wire_val, std::any::type_name::<Input2 #bridging_args>()));

                                let op_i2 = <#bridging_path as PortTryFrom2<Input2 #bridging_args>>::port_try_from2(i2)
                                    .expect("expected to be able to wire into port 2");

                                adapted_in[2] = Some(op_i2);
                            }

                            //----------------------------------------------------[port-3]
                            if let Some(wire_val) = input[3] {

                                let i3 = wire_val.port_try_into_dynamic::<Input3 #bridging_args>()
                                    .expect(&format!("could not unpack wire val {:#?} into i3 of ty={}", wire_val, std::any::type_name::<Input3 #bridging_args>()));

                                let op_i3 = <#bridging_path as PortTryFrom3<Input3 #bridging_args>>::port_try_from3(i3)
                                    .expect("expected to be able to wire into port 3");

                                adapted_in[3] = Some(op_i3);
                            }

                            let adapted_in_refs: [Option<&#bridging_path>; 4] = [
                                adapted_in[0].as_ref(),
                                adapted_in[1].as_ref(),
                                adapted_in[2].as_ref(),
                                adapted_in[3].as_ref(),
                            ];

                            <Self as hydro2_operator::Operator<#bridging_path>>::execute(
                                self,
                                adapted_in_refs,
                                &mut adapted_out
                            ).await?;

                            for i in 0..4 {
                                if let Some(val) = adapted_out[i].take() {
                                    output[i] = Some(#network_wire_enum_ident :: #new_io_ident(val));
                                }
                            }
                            Ok(())
                        }
                    }
        };
    };

    info!("[build_single_operator_impl] FINISH, expanded = {}", expanded.to_string());
    expanded
}

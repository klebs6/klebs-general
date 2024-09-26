#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;
#[macro_use] mod syn_imports; use syn_imports::*;

xp!{attribute_kind}
xp!{check_is_function_async}
xp!{ensure_item_has_no_test_attribute}
xp!{errors}
xp!{extract_all_attributes_except_test_attribute}
xp!{generate_new_block}
xp!{is_return_type_result}
xp!{is_should_panic_attr}
xp!{is_test_attribute}
xp!{result_handling}
xp!{tracing_setup_tokens}
xp!{should_panic_attr}
xp!{should_fail_attr}
xp!{traced_test}
xp!{wrap_async_test}
xp!{wrap_sync_test}
xp!{wrap_the_original_block}
xp!{test_builder}
xp!{sync_should_fail}
xp!{sync_should_pass}
xp!{async_should_fail}
xp!{async_should_pass}
xp!{original_block}
xp!{check_panic_message}
xp!{extract_traits}

#[proc_macro_attribute]
pub fn traced_test(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let item_fn = parse_macro_input!(item as ItemFn);

    // Parse the input tokens into a syntax tree
    let generator = match TracedTestGenerator::new(item_fn) {
        Ok(test) => test,
        Err(e)   => panic!("traced_test generator error: {:#?}", e),
    };

    let output = match generator.write() {
        Ok(output_fn) => output_fn,
        Err(e) => panic!("traced_test generator error: {:#?}", e),
    };

    output.into()
}

/// A no-op attribute to allow the compiler to recognize `#[should_fail]`.
#[proc_macro_attribute]
pub fn should_fail(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simply return the item unchanged
    item
}

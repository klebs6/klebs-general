#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]
#![feature(proc_macro_diagnostic)]
#![allow(unused_variables)]
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;
#[macro_use] mod syn_imports; use syn_imports::*;

xp!{async_should_fail}
xp!{async_should_pass}
xp!{attribute_kind}
xp!{check_is_function_async}
xp!{check_panic_message}
xp!{ensure_item_has_no_test_attribute}
xp!{errors}
xp!{extract_all_attributes_except_test_attribute}
xp!{generate_new_block}
xp!{check_returns_result}
xp!{return_type_tokens}
xp!{write_token_stream}
xp!{get_should_fail_attr}
xp!{is_should_panic_attr}
xp!{should_trace}
xp!{is_test_attribute}
xp!{original}
xp!{panic_handler}
xp!{parse_or_compile_error}
xp!{result_handling}
xp!{should_fail_attr}
xp!{should_panic_attr}
xp!{traced_test_attr}
xp!{sync_should_fail}
xp!{sync_should_pass}
xp!{test_builder}
xp!{traced_test}
xp!{tracing_setup_tokens}
xp!{use_statements}
xp!{wrap_async_test}
xp!{wrap_sync_test}
xp!{wrap_the_original_block}
xp!{backtrace_guard}
xp!{flush_logs_if_needed}
xp!{tracing_guard}
xp!{end_of_test_guard}

#[proc_macro_attribute]
pub fn traced_test(attr: TokenStream, item: TokenStream) -> TokenStream {

    let traced_test_attr = parse_macro_input!(attr as TracedTestAttr);
    let item_fn          = parse_macro_input!(item as ItemFn);

    let generator = match TracedTestGenerator::from_item_fn(item_fn, traced_test_attr) {
        Ok(test) => test,
        Err(e) => panic!("traced_test generator error: {:#?}", e),
    };

    let output = match generator.write_token_stream() {
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

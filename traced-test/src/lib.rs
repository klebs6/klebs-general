#![allow(dead_code)]
#![allow(unused_imports)]
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{check_is_function_async}
xp!{ensure_item_has_no_test_attribute}
xp!{errors}
xp!{extract_all_attributes_except_test_attribute}
xp!{is_return_type_result}
xp!{is_test_attribute}
xp!{should_panic}
xp!{traced_test}
xp!{generate_new_block}

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

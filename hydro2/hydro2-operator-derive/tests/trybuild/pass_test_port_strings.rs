// ---------------- [ File: hydro2-operator-derive/tests/trybuild/pass_test_port_strings.rs ]
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

/// We'll define a test struct with inputs = [i32, &str], outputs = [Vec<u8>].
/// Then we'll derive(Operator) for it and check the port type strings.
#[derive(NamedItem,Operator,Debug)]
#[operator(
    execute="my_execute",
    opcode="OpCode::TestOp",
    input0="i32",
    //TODO: should try this with &str and fix it
    input1="String",
    output0="Vec<u8>"
)]
pub struct MyTestOp {
    name: String,
}

impl MyTestOp {

    pub async fn my_execute(&self, _a: &i32, _b: &String) -> NetResult<Vec<u8>> {
        Ok(vec![1,2,3])
    }
}

/// Now test that input_port_type_str() / output_port_type_str() 
/// reflect the textual type strings exactly as expected.
#[tokio::main]
async fn main() {

    let op = MyTestOp { name: "test".to_string() };

    assert_eq!(op.input_count(), 2);
    assert_eq!(op.output_count(), 1);

    // The macro should produce match arms that return these exact type strings:
    assert_eq!(op.input_port_type_str(0), Some("i32"));
    assert_eq!(op.input_port_type_str(1), Some("alloc::string::String"));
    // out of range => None
    assert_eq!(op.input_port_type_str(2), None);
    assert_eq!(op.output_port_type_str(0), Some("alloc::vec::Vec<u8>"));
    // out of range => None
    assert_eq!(op.output_port_type_str(1), None);

    // Confirm that we can call op.execute(...) too:
    let mut output_slots = [None, None, None, None];

    let input0 = MyTestOpIO::Input0(123i32);
    let input1 = MyTestOpIO::Input1("hello".to_string());

    let input_slots = [
        Some(&input0),
        Some(&input1),
        None,
        None,
    ];
    let res = op.execute(input_slots, &mut output_slots).await;
    assert!(res.is_ok());
    // The single output (index 0) should be Some(MyTestOpIO::Output0(_)).
    match &output_slots[0] {
        Some(MyTestOpIO::Output0(v)) => assert_eq!(v, &vec![1,2,3]),
        other => panic!("Unexpected output[0]: {:?}", other),
    }
}

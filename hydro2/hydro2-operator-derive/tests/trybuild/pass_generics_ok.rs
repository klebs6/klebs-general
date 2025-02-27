// ---------------- [ File: hydro2-operator-derive/tests/trybuild/pass_generics_ok.rs ]
// tests/trybuild/pass_generics_ok.rs

#![allow(unused)]
use std::fmt::{Debug,Display};
use std::ops::{Add, Mul, Sub};

use hydro2_operator::*;         // wherever you define #[derive(Operator)]
use hydro2_operator_derive::*;         // wherever you define #[derive(Operator)]
use named_item::*;
use named_item_derive::*;

#[derive(NamedItem,Debug,Operator)]
#[operator(
    execute="emit",
    opcode="OpCode::ConstantOp",
    output0="U"
)]
pub struct ConstantOp<U>
where U: Display + Copy + Send + Sync + Debug
{
    pub name:  String,
    pub value: U,
}

impl<U> ConstantOp<U>
where U: Display + Copy + Send + Sync + Debug
{
    async fn emit(&self) -> NetResult<U> {
        Ok(self.value)
    }
}

#[derive(NamedItem,Debug,Operator)]
#[operator(
    execute="double_to_tri",
    opcode="OpCode::DoubleToTriOp",
    input0="T", input1="T",
    output0="T", output1="T", output2="T"
)]
pub struct DoubleToTriOp<T>
where T: Copy + Add<Output=T> + Mul<Output=T> + Sub<Output=T> + Send + Sync + Debug
{
    name:  String,
    // Must store T or PhantomData<T> so it's genuinely used:
    _0: std::marker::PhantomData<T>,
}

impl<T> DoubleToTriOp<T>
where T: Copy + Add<Output=T> + Mul<Output=T> + Sub<Output=T> + Send + Sync + Debug
{
    async fn double_to_tri(&self, lhs: &T, rhs: &T) -> NetResult<(T,T,T)> {
        Ok(((*lhs + *rhs), (*lhs - *rhs), (*lhs * *rhs)))
    }
}

// The presence of these types ensures we attempt expansions:
fn main() {
    let _cst = ConstantOp { name: "OK".to_string(), value: 42 };
    let _dto = DoubleToTriOp::<f64> { 
        name: "Ok".to_string(), 
        _0: std::marker::PhantomData,
    };
}

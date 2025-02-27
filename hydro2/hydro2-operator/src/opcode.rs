// ---------------- [ File: hydro2-operator/src/opcode.rs ]
crate::ix!();

pub trait OpCode: Send + Sync {
    fn val(&self) -> u64;
}

/// High-level codes for each stage/operator in the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BasicOpCode {
    Foo,
    Nothing,
    Sink,
    TestOp,
    MultiThing,
    NoOp,
    FailingOp,
    StreamTestOp,
    SingleValueTestOp,
    AddOp,
    MultiplyOp,
    ToStringOp,
    ConstantOp,
    IncrementOperator,
    SingleChannelPassthrough,
    DoubleToTriOp,
    DoubleOp,
    DoubleToTriTwoGenericsOp,
    Merge2Op,
    DoubleOutOp,
    QuadToQuadOp,
    SingleToTriOp,
    TriToQuadOp,
    TriToSingleOp,
    SplitAndDoubleOp,
}

unsafe impl Send for BasicOpCode {}
unsafe impl Sync for BasicOpCode {}

impl OpCode for BasicOpCode {
    fn val(&self) -> u64 {
        use BasicOpCode::*;
        match self {
            Foo                      => 0,
            Nothing                  => 1,
            Sink                     => 2,
            TestOp                   => 3,
            MultiThing               => 4,
            NoOp                     => 5,
            FailingOp                => 6,
            StreamTestOp             => 7,
            SingleValueTestOp        => 8,
            AddOp                    => 9,
            MultiplyOp               => 10,
            ToStringOp               => 11,
            ConstantOp               => 12,
            IncrementOperator        => 13,
            SingleChannelPassthrough => 14,
            DoubleToTriOp            => 15,
            DoubleOp                 => 16,
            DoubleToTriTwoGenericsOp => 17,
            Merge2Op                 => 18,
            DoubleOutOp              => 19,
            QuadToQuadOp             => 20,
            SingleToTriOp            => 21,
            TriToQuadOp              => 22,
            TriToSingleOp            => 23,
            SplitAndDoubleOp         => 24,
        }
    }
}

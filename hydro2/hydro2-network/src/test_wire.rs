crate::ix!();

#[derive(NetworkWire,Default,PartialEq,Eq,Debug,Clone)]
#[available_operators(
    op="AddOp",
    op="ConstantOp<T>",
    op="DoubleOutOp",
    op="FailingOperator",
    op="IncrementOperator",
    op="Merge2Op",
    op="MultiplyOp",
    op="NoOpOperator",
    op="SingleChannelPassthroughOperator<T>",
    op="SingleValOp",
    op="SplitAndDoubleOp",
    op="StreamyOperator<T>",
    op="SinkOperator<T>",
)]
pub struct TestWire<T: Zero + Display + Copy + Debug + Send + Sync + PartialEq + Eq> {
    _0: PhantomData<T>,
}

#[macro_export]
macro_rules! test_wire_port0_into {
    ($x:ident => $ty:ty) => {
        <TestWireIO<i32> as PortTryInto0<$ty>>::port_try_into0($x).expect("expected to be able to wire into type")
    }
}

#[macro_export]
macro_rules! test_wire_port1_into {
    ($x:ident => $ty:ty) => {
        <TestWireIO<i32> as PortTryInto1<$ty>>::port_try_into1($x).expect("expected to be able to wire into type")
    }
}

#[macro_export]
macro_rules! test_wire_port2_into {
    ($x:ident => $ty:ty) => {
        <TestWireIO<i32> as PortTryInto2<$ty>>::port_try_into2($x).expect("expected to be able to wire into type")
    }
}

#[macro_export]
macro_rules! test_wire_port3_into {
    ($x:ident => $ty:ty) => {
        <TestWireIO<i32> as PortTryInto3<$ty>>::port_try_into3($x).expect("expected to be able to wire into type")
    }
}

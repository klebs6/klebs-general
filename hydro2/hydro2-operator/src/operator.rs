// ---------------- [ File: hydro2-operator/src/operator.rs ]
crate::ix!();

pub type NetworkNodeIoChannelArray<NetworkItem>              = [Option<Arc<AsyncRwLock<NetworkItem>>>;         4];
pub type NetworkNodeIoChannelValues<NetworkItem>             = [Option<NetworkItem>;                           4];
pub type NetworkNodeIoChannelReadGuardArray<'a,NetworkItem>  = [Option<AsyncRwLockReadGuard<'a,NetworkItem>>;  4];
pub type NetworkNodeIoChannelWriteGuardArray<'a,NetworkItem> = [Option<AsyncRwLockWriteGuard<'a,NetworkItem>>; 4];

/// A trait that describes a single operator within the network.
/// Each operator is responsible for processing input buffers
/// to produce output buffers.
#[async_trait]
pub trait Operator<NetworkItem>: Debug + Named + Send + Sync 
where NetworkItem: Debug + Send + Sync
{
    /// Returns the operation code, which can be used to inform
    /// specialized handling or diagnostics.
    fn opcode(&self) -> OpCode;

    /// How many actual inputs does this operator need?
    fn input_count(&self) -> usize;

    /// How many outputs does this operator produce?
    fn output_count(&self) -> usize;

    /// used by the network! dag compiler to verify that the input port of one operator is
    /// compatible with the data flowing into it from the output port of another operator
    fn input_port_type_str(&self, port: usize) -> Option<&'static str>;

    /// used by the network! dag compiler to verify that the output port of one operator is
    /// compatible with the data required by the input port of its downstream operator
    fn output_port_type_str(&self, port: usize) -> Option<&'static str>;

    /// used by the network! dag compiler to verify that this input port needs an output connection
    fn input_port_connection_required(&self, port: usize) -> bool;

    /// used by the network! dag compiler to verify that this output port needs an input connection
    fn output_port_connection_required(&self, port: usize) -> bool;

    /// The big 4×4 method:
    /// You receive up to 4 inputs and must fill up to 4 outputs.
    async fn execute(
        &self,
        input:  [Option<&NetworkItem>; 4],
        output: &mut [Option<NetworkItem>; 4],
    ) -> NetResult<()>;
}

/// A local trait to convert any `T` that implements `Operator` into `Arc<dyn Operator>`.
pub trait IntoArcOperator<NetworkItem> {
    fn into_arc_operator(self) -> Arc<dyn Operator<NetworkItem>>;
}

impl<T,NetworkItem> IntoArcOperator<NetworkItem> for T
where
    T: Operator<NetworkItem> + 'static,
    NetworkItem: Debug + Send + Sync,
{
    fn into_arc_operator(self) -> Arc<dyn Operator<NetworkItem>> {
        Arc::new(self)
    }
}

/// The trait describing up to 4 inputs and 4 outputs for an operator.
/// Each `#[derive(Operator)]` implementation will provide a hidden struct
/// implementing these 8 associated types.
pub trait OperatorSignature {
    type Input0; 
    type Input1; 
    type Input2; 
    type Input3;
    type Output0; 
    type Output1; 
    type Output2; 
    type Output3;
}

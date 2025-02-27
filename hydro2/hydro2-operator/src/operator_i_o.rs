// ---------------- [ File: hydro2-operator/src/operator_i_o.rs ]
crate::ix!();

#[async_trait]
pub trait OperatorI_O<NetworkItem, const I: usize, const O: usize>: Debug + Named + Send + Sync
where
    NetworkItem: Debug + Send + Sync,
{
    fn opcode(&self) -> OpCode;

    /// The user’s main method for exactly I inputs and O outputs.
    async fn execute(
        &self,
        input:  &[Option<&NetworkItem>; I],     // by reference to an array, not by value
        output: &mut [Option<NetworkItem>; O], // also by reference
    ) -> NetResult<()>;
}

fn slice_to_subarray_ref<T, const N: usize>(slice: &[T]) -> &[T; N] {
    assert_eq!(slice.len(), N);
    unsafe { &*(slice.as_ptr() as *const [T; N]) }
}

fn slice_to_subarray_mut_ref<T, const N: usize>(slice: &mut [T]) -> &mut [T; N] {
    assert_eq!(slice.len(), N);
    unsafe { &mut *(slice.as_mut_ptr() as *mut [T; N]) }
}

/// Blanket-impl the 4×4 `Operator<NetworkItem>` for any type
/// that implements `OperatorI_O<NetworkItem, I, O>`.
#[async_trait]
impl<T, NetworkItem> Operator<NetworkItem> for T
where
    T: OperatorI_O<NetworkItem, 1, 1>,
    NetworkItem: Debug + Send + Sync,
{
    fn opcode(&self) -> OpCode {
        <Self as OperatorI_O<NetworkItem, 1, 1>>::opcode(self)
    }

    fn input_count(&self) -> usize {
        1
    }

    fn output_count(&self) -> usize {
        1
    }

    async fn execute4(
        &self,
        input4:  [Option<&NetworkItem>; 4],
        output4: &mut [Option<NetworkItem>; 4],
    ) -> NetResult<()> {
        // We treat input4 as a slice of length 4:
        let input_slice: &[Option<&NetworkItem>] = &input4;
        // Then we “borrow” the first I elements as &[Option<&NetworkItem>; I].
        let input_i: &[Option<&NetworkItem>; 1] = slice_to_subarray_ref(&input_slice[..1]);

        let output_slice: &mut [Option<NetworkItem>] = &mut output4[..];
        let output_o: &mut [Option<NetworkItem>; 1] = slice_to_subarray_mut_ref(&mut output_slice[..1]);

        self.execute(input_i, output_o).await
    }
}

#[async_trait]
impl<T, NetworkItem> Operator<NetworkItem> for T
where
    T: OperatorI_O<NetworkItem, 2, 2>,
    NetworkItem: Debug + Send + Sync,
{
    fn opcode(&self) -> OpCode {
        <Self as OperatorI_O<NetworkItem, 2, 2>>::opcode(self)
    }

    fn input_count(&self) -> usize {
        2
    }

    fn output_count(&self) -> usize {
        2
    }

    async fn execute4(
        &self,
        input4:  [Option<&NetworkItem>; 4],
        output4: &mut [Option<NetworkItem>; 4],
    ) -> NetResult<()> {
        // We treat input4 as a slice of length 4:
        let input_slice: &[Option<&NetworkItem>] = &input4;
        // Then we “borrow” the first I elements as &[Option<&NetworkItem>; I].
        let input_i: &[Option<&NetworkItem>; 2] = slice_to_subarray_ref(&input_slice[..2]);

        let output_slice: &mut [Option<NetworkItem>] = &mut output4[..];
        let output_o: &mut [Option<NetworkItem>; 2] = slice_to_subarray_mut_ref(&mut output_slice[..2]);

        self.execute(input_i, output_o).await
    }
}

// If your Rust >= 1.72 stable, you can do “trait alias”:
pub trait Operator0_0<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 0, 0>;
pub trait Operator0_1<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 0, 1>;
pub trait Operator0_2<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 0, 2>;
pub trait Operator0_3<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 0, 3>;
pub trait Operator0_4<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 0, 4>;

pub trait Operator1_0<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 1, 0>;
pub trait Operator1_1<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 1, 1>;
pub trait Operator1_2<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 1, 2>;
pub trait Operator1_3<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 1, 3>;
pub trait Operator1_4<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 1, 4>;

pub trait Operator2_0<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 2, 0>;
pub trait Operator2_1<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 2, 1>;
pub trait Operator2_2<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 2, 2>;
pub trait Operator2_3<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 2, 3>;
pub trait Operator2_4<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 2, 4>;

pub trait Operator3_0<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 3, 0>;
pub trait Operator3_1<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 3, 1>;
pub trait Operator3_2<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 3, 2>;
pub trait Operator3_3<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 3, 3>;
pub trait Operator3_4<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 3, 4>;

pub trait Operator4_0<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 4, 0>;
pub trait Operator4_1<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 4, 1>;
pub trait Operator4_2<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 4, 2>;
pub trait Operator4_3<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 4, 3>;
pub trait Operator4_4<NetworkItem: Debug + Send + Sync> = OperatorI_O<NetworkItem, 4, 4>;

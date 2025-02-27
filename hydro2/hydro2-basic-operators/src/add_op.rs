// ---------------- [ File: src/add_op.rs ]
crate::ix!();

#[derive(NamedItem,Operator,Debug)]
#[operator(
    execute="add", 
    opcode="BasicOpCode::AddOp", 
    input0="i32", 
    output0="i32"
)]
pub struct AddOp {
    name:   String,
    addend: i32,
}

impl AddOp {
    pub fn new(a: i32) -> Self {
        let name = format!("AddOp(+{})", a);
        Self { addend: a, name }


    }
    async fn add(&self, input0: &i32) -> NetResult<i32> {
        info!("OPERATOR running AddOp with addend: {}", self.addend);
        Ok(*input0 + self.addend)
    }
}

#[cfg(test)]
mod add_op_tests {

    use super::*;

    #[tokio::test]
    async fn test_add_op_simple() -> Result<(), NetworkError> {
        let add = AddOp::new(100);
        let input = [
            Some(&AddOpIO::Input0(42)),
            None,
            None,
            None,
        ];
        let mut out = [None, None, None, None];

        add.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(AddOpIO::Output0(142)));
        Ok(())
    }

    #[tokio::test]
    async fn test_add_op_negative() -> Result<(), NetworkError> {
        let add = AddOp::new(-20);
        let input = [Some(&AddOpIO::Input0(100)), None, None, None];
        let mut out = [None,None,None,None];
        add.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(AddOpIO::Output0(80)));
        Ok(())
    }

    #[tokio::test]
    async fn test_add_op_multiple_calls() -> Result<(), NetworkError> {
        let add = AddOp::new(5);
        for x in [0_i32, 10, 100, -10] {


            let i0 = AddOpIO::Input0(x);
            let input = [Some(&i0), None, None, None];
            let mut out = [None,None,None,None];
            add.execute(input, &mut out).await?;
            assert_eq!(out[0], Some(AddOpIO::Output0(x+5)));
        }
        Ok(())
    }
}

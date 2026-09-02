use error_tree::error_tree;

#[derive(Debug)]
struct PayloadError;

error_tree! {
    enum RootError {
        Structured {
            payload: PayloadError
        },
    }
}

trait FromEdgeWitness {}

impl<T> FromEdgeWitness for T
where
    T: From<PayloadError>
{}

impl FromEdgeWitness for RootError {}

fn assert_no_generated_from_edge<T: FromEdgeWitness>() {}

fn main() {
    assert_no_generated_from_edge::<RootError>();
}

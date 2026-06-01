use error_tree::error_tree;

#[derive(Debug)]
pub struct LeafPayload;

error_tree! {
    pub enum RootError {
        Child(ChildError),
    }

    pub enum ChildError {
        Leaf(LeafPayload),
    }
}

fn main() {
    let payload = LeafPayload;
    let _child: ChildError = payload.into();

    let payload = LeafPayload;
    let _root: RootError = payload.into();
}

crate::ix!();

pub trait ErrorTreeVisitor {

    fn visit_error_enum(&mut self, e: &ErrorEnum);
    // other methods as needed
}

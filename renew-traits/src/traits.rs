crate::ix!();

pub trait ExtendWith<Item> {

    type Error;

    fn extend_with(&mut self, item: &Item) 
        -> Result<(),Self::Error>;
}

pub trait FillToLenWithItems {

    type Item;

    fn fill_to_len(&mut self, len: usize, items: Vec<Self::Item>);
}

pub trait ReinitWithLen {

    fn reinit(&mut self, len: usize);
}

pub trait FillWith {

    type Item;

    fn fill(&mut self, val: Self::Item);
}

pub trait InitInternals {

    type Error;

    fn init_internals(&mut self) 
    -> Result<(),Self::Error>;
}

pub trait InitWithSize {

    fn init_size(&mut self, size: usize);
}

pub trait Clear {

    fn clear(&mut self);
}

pub trait CreateNamedEmpty {

    fn empty(name: &str) -> Self;
}

pub trait CreateEmpty {

    fn empty() -> Self;
}

pub trait ResetWith<Input> {

    fn reset_with(&mut self, g: &Input);
}

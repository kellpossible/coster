pub trait StoreEvent {
    fn none() -> Self;
    fn is_none(&self) -> bool;
}

impl StoreEvent for () {
    fn none() -> Self {
        ()
    }

    fn is_none(&self) -> bool {
        true
    }
}

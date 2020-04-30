pub trait Field {
    fn label(&self) -> String;
}

impl Field for &str {
    fn label(&self) -> String {
        self.to_string()
    }
}

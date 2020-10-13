pub struct DisplayDriver {
    pub hello: bool,
}

impl Default for DisplayDriver {
    fn default() -> Self {
        Self::new()
    }
}
impl DisplayDriver {
    pub fn new() -> DisplayDriver {
        DisplayDriver { hello: false }
    }
}

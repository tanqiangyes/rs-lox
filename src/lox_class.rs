#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoxClass {
    name: String,
}

impl std::string::ToString for LoxClass {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

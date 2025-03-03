pub trait Role: Send + Sync {
    fn is_authenticated(&self) -> bool;
}

pub struct DefaultRole {}

impl Role for DefaultRole {
    fn is_authenticated(&self) -> bool { false }
}

impl DefaultRole {
    pub fn new() -> Self { DefaultRole {} }
}
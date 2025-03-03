pub trait Role: Send + Sync {
    fn is_authenticated(&self) -> bool;
}

pub struct CharacterRole {
    pub account_id: u64,
    pub character_id: u64,
}

impl CharacterRole {
    pub fn new() -> CharacterRole {
        CharacterRole {
            account_id: 0,
            character_id: 0,
        }
    }
}

impl Role for CharacterRole {
    fn is_authenticated(&self) -> bool { self.account_id != 0 }
}

pub struct AdminRole {}

impl Role for AdminRole {
    fn is_authenticated(&self) -> bool { true }
}
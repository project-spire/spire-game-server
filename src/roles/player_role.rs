use crate::core::role::Role;

pub struct PlayerRole {
    pub account_id: u64,
    pub character_id: u64,
}

impl Role for PlayerRole {
    fn is_authenticated(&self) -> bool { self.account_id != 0 }
}

impl PlayerRole {
    pub fn new() -> PlayerRole {
        PlayerRole {
            account_id: 0,
            character_id: 0,
        }
    }
}

pub struct CheatPlayerRole {
    pub account_id: u64,
    pub character_id: u64,
}

impl Role for CheatPlayerRole {
    fn is_authenticated(&self) -> bool { true }
}

impl CheatPlayerRole {
    pub fn new() -> CheatPlayerRole {
        CheatPlayerRole {
            account_id: 0,
            character_id: 0,
        }
    }
}
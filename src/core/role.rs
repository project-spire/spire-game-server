pub enum Role {
    Default,
    Player(Account),
    CheatPlayer(Account),
    Admin,
}

pub struct Account {
    pub account_id: u64,
    pub character_id: u64,
}

impl Account {
    pub fn new(account_id: u64, character_id: u64) -> Self {
        Account { account_id, character_id }
    }
}

impl Default for Account {
    fn default() -> Self { Account::new(0, 0) }
}
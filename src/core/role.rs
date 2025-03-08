use std::fmt;

#[derive(Debug)]
pub enum Role {
    Player(Account),
    CheatPlayer(Account),
    Admin,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Player(account) => write!(f, "Role(Player), {}", account),
            Role::CheatPlayer(account) => write!(f, "Role(CheatPlayer), {}", account),
            Role::Admin => write!(f, "Role(Admin)"),
        }
    }
}

#[derive(Debug)]
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

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Account( aid: {}, cid: {} )", self.account_id, self.character_id)
    }
}
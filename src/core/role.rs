pub enum Role {
    Default,
    Player(PlayerRole),
    CheatPlayer(PlayerRole),
    Admin,
}

pub struct PlayerRole {
    pub account_id: u64,
    pub character_id: u64,
}

impl PlayerRole {
    pub fn new(account_id: u64, character_id: u64) -> Self {
        PlayerRole { account_id, character_id }
    }
}

impl Default for PlayerRole {
    fn default() -> Self { PlayerRole::new(0, 0) }
}
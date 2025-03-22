use bevy_ecs::prelude::*;
use crate::character::movement::MovementController;
use crate::character::stat::MobilityStats;
use crate::core::session::Session;
use crate::physics::object::Transform;

#[derive(Debug)]
pub enum Privilege {
    None,
    Manager,
}

#[derive(Component, Debug)]
pub struct Account {
    pub account_id: u64,
    pub character_id: u64,
    pub privilege: Privilege,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    // character
    pub transform: Transform,
    pub movement_controller: MovementController,
    pub mobility: MobilityStats,

    // network
    pub account: Account,
    pub session: Session,
}

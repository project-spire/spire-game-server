use std::sync::Arc;
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


impl PlayerBundle {
    pub async fn load(account: Account, session: Session) -> (Arc<Self>, u64) {
        let mut last_room: u64 = 0;

        (Arc::new(PlayerBundle {
            transform: Transform::default(),
            movement_controller: MovementController::default(),

            account,
            session
        }), last_room)
    }
}
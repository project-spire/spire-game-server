use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct WorldTime {
    pub now: std::time::Instant,
    pub dt: std::time::Duration,
}
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Abilities {
    // Core stats
    strength: u16,
    dexterity: u16,
    constitution: u16,
    intelligence: u16,

    // Optional stats
    faith: Option<u16>,
}

#[derive(Component)]
pub struct PlayerStats {
    level: u16,
    exp: u32,
    max_exp: u32,
}

#[derive(Component)]
pub struct MobilityStats {
    pub speed: f32,
    base_speed: f32,
}

#[derive(Component)]
pub struct CombatStats {
    pub attack: u32,
    base_attack: u32,
}

#[derive(Component)]
pub struct CraftingStats {

}
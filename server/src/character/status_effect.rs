use bevy_ecs::prelude::*;
use std::time::Instant;

pub enum StatusEffectType {
    Buff,
    Debuff,
    Passive,
    Curse,
}

pub enum StatusEffect {
    Stun,
    Slow { modifier: u8 },
}

#[derive(Component)]
pub struct StatusEffectController {
    pub temporary_effects: Vec<(StatusEffect, Instant)>,
    pub permanent_effects: Vec<StatusEffect>,
}

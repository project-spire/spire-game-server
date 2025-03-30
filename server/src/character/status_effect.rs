use bevy_ecs::prelude::*;
use std::time::Instant;
use macros::StatusEffect;

pub enum StatusEffectKind {
    Buff,
    Debuff,
    Passive,
    Curse,
}

#[derive(StatusEffect)]
pub enum StatusEffect {
    #[debuff] Stun,
    // #[debuff] Slow { modifier: u8 },
    // #[buff] Haste { modifier: u8 },
}

#[derive(Component)]
pub struct StatusEffectController {
    pub temporary_effects: Vec<(StatusEffect, Instant)>,
    pub permanent_effects: Vec<StatusEffect>,
}

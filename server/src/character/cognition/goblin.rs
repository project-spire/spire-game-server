use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct GoblinWarriorBrain {}

#[derive(Component)]
pub struct GoblinArcherBrain {}

pub fn update_goblin_warrior(goblins: Query<&GoblinWarriorBrain>) {

}

pub fn update_goblin_archer(goblins: Query<&GoblinArcherBrain>) {

}
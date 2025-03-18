use bevy_ecs::prelude::*;

#[derive(Bundle)]
pub struct PlayerBundle {
    //TODO
}

pub fn extract(world: &mut World, entity: Entity) -> Option<PlayerBundle> {
    if !world.entities().contains(entity) {
        return None;
    }

    let mut entity_ref = world.entity_mut(entity);
    entity_ref.take::<PlayerBundle>()
}
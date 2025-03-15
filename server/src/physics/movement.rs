use bevy_ecs::prelude::*;
use crate::physics::object::DynamicBody;

pub fn dynamic_movement(mut dynamic_bodies: Query<&mut DynamicBody>) {
    dynamic_bodies.iter_mut().for_each(|mut body| {
        let position = body.position + body.velocity;
        body.position = position;
    });
}
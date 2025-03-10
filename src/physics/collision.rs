use bevy_ecs::prelude::*;
use nalgebra_glm::Vec2;

pub enum CollisionShape {
    Dot { v: Vec2 },
    Rectangle { x: f32, y: f32 },
    Circle { radius: f32 },
}

#[derive(Component)]
pub struct Collision {
    pub a: Entity,
    pub b: Entity,
}

pub fn is_colliding(a: &CollisionShape, b: &CollisionShape) -> bool {
    use CollisionShape::*;

    // match (a, b) {
    //     (Dot { v: va }, Dot { v: vb }) => va == vb,
    // }
}

use crate::physics::collision::CollisionShape;
use bevy_ecs::component::Component;
use nalgebra::{Point2, UnitVector2, Vector2};
use std::ops::{Deref, DerefMut};

#[derive(Component)]
pub struct Position(Point2<f32>);

impl Deref for Position {
    type Target = Point2<f32>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

#[derive(Component)]
pub struct Transform {
    pub position: Point2<f32>,
    pub rotation: UnitVector2<f32>,
    pub velocity: Vector2<f32>,
}

#[derive(Component)]
pub struct StaticBody {
    pub shape: CollisionShape,
}

#[derive(Component)]
pub struct KinematicBody {
    pub shape: CollisionShape,
}

#[derive(Component)]
pub struct TriggerBody {
    pub shape: CollisionShape,
}

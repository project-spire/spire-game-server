use crate::physics::collision::CollisionShape;
use bevy_ecs::component::Component;
use nalgebra::{ Point2, Vector2 };

pub struct PhysicsMaterial {
    pub absorbent: bool,
    pub bounce: f32,
    pub friction: f32,
    pub rough: bool
}

impl PhysicsMaterial {
    pub fn new(absorbent: bool, bounce: f32, friction: f32, rough: bool) -> Self {
        Self {
            absorbent,
            bounce: bounce.clamp(0.0, 1.0),
            friction: friction.clamp(0.0, 1.0),
            rough
        }
    }
}

#[derive(Component)]
pub struct TriggerBody {
    pub position: Point2<f32>,
    pub shape: CollisionShape,
}

#[derive(Component)]
pub struct StaticBody {
    pub position: Point2<f32>,
    pub shape: CollisionShape,
    pub material: Option<PhysicsMaterial>,
}

#[derive(Component)]
pub struct KinematicBody {
    pub position: Point2<f32>,
    pub shape: CollisionShape,
    pub material: Option<PhysicsMaterial>,
}

#[derive(Component)]
pub struct DynamicBody {
    pub position: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub shape: CollisionShape,
    pub material: Option<PhysicsMaterial>,
}


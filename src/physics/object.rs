use crate::physics::collision::CollisionShape;
use bevy_ecs::component::Component;
use nalgebra_glm::Vec2;

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

    pub fn set_bounce(&mut self, bounce: f32) {
        self.bounce = bounce.clamp(0.0, 1.0);
    }

    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction.clamp(0.0, 1.0);
    }
}

#[derive(Component)]
pub struct TriggerBody {
    pub position: Vec2,
    pub shape: CollisionShape,
}

#[derive(Component)]
pub struct StaticBody {
    pub position: Vec2,
    pub shape: CollisionShape,
    pub material: Option<PhysicsMaterial>,
}

#[derive(Component)]
pub struct DynamicBody {
    pub position: Vec2,
    pub velocity: Vec2,
    pub shape: CollisionShape,
    pub material: Option<PhysicsMaterial>,
}
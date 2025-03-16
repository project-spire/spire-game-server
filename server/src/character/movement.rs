use bevy_ecs::prelude::*;
use crate::character::status::Status;
use crate::physics::object::Transform;
use crate::world::time::WorldTime;
use nalgebra::{Point2, UnitVector2};

use crate::character::movement::MovementState::*;
use crate::character::movement::MovementMode::*;
use crate::character::movement::MovementCommand::*;

type Transition = (MovementState, std::time::Instant);

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum MovementState {
    Idle,
    Walking,
    Running,
    Rolling
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum MovementMode {
    Standing,
    Crouching,
    Crawling,
    Swimming,
    Flying,
}

pub enum MovementCommand {
    // State changes
    Halt,
    Walk { direction: UnitVector2<f32> },
    Run { direction: UnitVector2<f32> },
    Roll { direction: UnitVector2<f32> },

    // Mode changes
    Stand,
    Crouch,
    Crawl,
    Swim,
    Fly,

    // etc
    Teleport { position: Point2<f32>, forced: bool },
}

#[derive(Component)]
pub struct MovementController {
    state: MovementState,
    mode: MovementMode,
    commands: Vec<MovementCommand>,
    transition: Option<Transition>,

    speed: f32,
    base_speed: f32,
}

pub fn update(
    mut controllers: Query<(&mut MovementController, &mut Transform, Option<&Status>)>,
    time: Res<WorldTime>,
) {
    controllers.iter_mut().for_each(|(mut controller, mut transform, status)| {
        if let Some(transition) = controller.transition.take() {
            handle_transition(transition, &mut controller, &time);
        }

        let commands: Vec<_> = controller.commands.drain(..).collect();
        for command in commands {
            handle_command(command, &mut controller, &mut transform, status, &time);
        }

        handle_movement(&controller, &mut transform, &time);
    })
}

fn handle_transition(
    transition: Transition,
    controller: &mut MovementController,
    time: &Res<WorldTime>,
) {
    let (state, then) = transition;

    if time.now < then {
        controller.transition = Some((state, then)); // give back
        return;
    }

    controller.state = state;
}

fn handle_command(
    command: MovementCommand,
    controller: &mut MovementController,
    transform: &mut Transform,
    status: Option<&Status>,
    time: &Res<WorldTime>,
) {
    match command {
        Halt => if controller.state == Walking || controller.state == Running {
            controller.state = Idle;
        }
        Walk { direction } => {
            //TODO: Check status
            controller.state = Walking;
            transform.rotation = direction;
        }
        Run { direction } => {
            //TODO: Check status
            controller.state = Running;
            transform.rotation = direction;
        }
        Roll { direction } => {
            //TODO: Check status
            //TODO: Set rolling expiration as transition
            controller.state = Rolling;
            transform.rotation = direction;
        }
        Stand => {
            //TODO: Check status
            controller.mode = Standing;
        }
        Crouch => {
            //TODO: Check status
            controller.mode = Crouching;
        }
        Crawl => {
            //TODO: Check status
            controller.mode = Crawling;
        }
        Swim => {
            //TODO: Check status
            controller.mode = Swimming;
        }
        Fly => {
            //TODO: Check status
            controller.mode = Flying;
        }
        Teleport { position, forced } => {
            if forced {
                //TODO: Check if movable to the position
                transform.position = position;
                return;
            }

            //TODO: Check status
            transform.position = position;
        }
    }
}

fn handle_movement(
    controller: &MovementController,
    transform: &mut Transform,
    time: &Res<WorldTime>,
) {
    //TODO: Extract to config?
    const WALK_MULTIPLIER: f32 = 1.0;
    const RUN_MULTIPLIER: f32 = 1.5;

    const STAND_MULTIPLIER: f32 = 1.0;
    const CROUCH_MULTIPLIER: f32 = 0.6;
    const CRAWL_MULTIPLIER: f32 = 0.3;

    if controller.state == Idle {
        return;
    }

    if controller.state == Rolling {
        //TODO
        return;
    }

    let mut speed = controller.speed;
    speed *= match controller.state {
        Walking => WALK_MULTIPLIER,
        Running => RUN_MULTIPLIER,
        _ => panic!(),
    };
    speed *= match controller.mode {
        Standing => STAND_MULTIPLIER,
        Crouching => CROUCH_MULTIPLIER,
        Crawling => CRAWL_MULTIPLIER,
        _ => 1.0,
    };

    let vector = speed * (time.dt.as_millis() as f32) * (*transform.rotation);
    transform.position += vector;
}
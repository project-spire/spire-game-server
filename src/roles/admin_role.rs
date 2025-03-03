use crate::core::role::Role;

pub struct AdminRole {}

impl Role for AdminRole {
    fn is_authenticated(&self) -> bool { true }
}
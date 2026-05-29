//! The `role` receiver — role-bound-NPC reads. Methods filled in Task 4 (fan-out).
//!
//! Unlike `m`/`f`, every method takes the role id as its first argument
//! (`role.getName("ROLE_X")`, `role.hasFlag("ROLE_X", "flag")`).

#[derive(Clone)]
pub struct Role;

pub fn register(engine: &mut rhai::Engine) {
    engine.register_type::<Role>();
}

//! The `m` receiver — active-male-NPC reads. Methods filled in Task 4 (fan-out).

#[derive(Clone)]
pub struct M;

pub fn register(engine: &mut rhai::Engine) {
    engine.register_type::<M>();
}

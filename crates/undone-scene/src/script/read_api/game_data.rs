//! The `gd` receiver — game-data reads. Methods filled in Task 4 (fan-out).

#[derive(Clone)]
pub struct Gd;

pub fn register(engine: &mut rhai::Engine) {
    engine.register_type::<Gd>();
}

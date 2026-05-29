//! The `f` receiver — active-female-NPC reads. Methods filled in Task 4 (fan-out).

#[derive(Clone)]
pub struct F;

pub fn register(engine: &mut rhai::Engine) {
    engine.register_type::<F>();
}

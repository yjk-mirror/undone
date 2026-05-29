//! The `scene` receiver — scene-local flag reads. Methods filled in Task 4 (fan-out).

#[derive(Clone)]
pub struct Scene;

pub fn register(engine: &mut rhai::Engine) {
    engine.register_type::<Scene>();
}

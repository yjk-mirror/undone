use floem::reactive::{RwSignal, SignalGet};

#[cfg(test)]
use floem::reactive::Scope;

#[cfg(test)]
pub(crate) fn get_or<T>(signal: RwSignal<T>, default: T) -> T
where
    T: Clone + 'static,
{
    signal.try_get().unwrap_or(default)
}

pub(crate) fn get_or_default<T>(signal: RwSignal<T>) -> T
where
    T: Clone + Default + 'static,
{
    signal.try_get().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_returns_value_while_signal_is_alive() {
        let scope = Scope::new();
        let signal = scope.create_rw_signal(42_u32);

        assert_eq!(get_or(signal, 0), 42);
    }

    #[test]
    fn get_or_returns_fallback_after_signal_is_disposed() {
        let scope = Scope::new();
        let signal = scope.create_rw_signal(42_u32);

        scope.dispose();

        assert_eq!(get_or(signal, 7), 7);
    }

    #[test]
    fn get_or_default_returns_default_after_signal_is_disposed() {
        let scope = Scope::new();
        let signal = scope.create_rw_signal(String::from("alive"));

        scope.dispose();

        assert_eq!(get_or_default(signal), String::new());
    }
}

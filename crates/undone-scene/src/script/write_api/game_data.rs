//! `gd.*` effect mutators — game-data writes.
//!
//! Each method mirrors an `apply_effect` arm in `effects.rs` that targets
//! `world.game_data`, wrapped in `with_write_ctx` for continue-on-error.
//! Methods are added to the `Gd` handle defined in `read_api::game_data`
//! (Rust allows inherent impls across modules in a crate).

use crate::effects::EffectError;
use crate::script::context::with_write_ctx;
use crate::script::read_api::game_data::Gd;

impl Gd {
    fn set_game_flag(&mut self, flag: &str) {
        with_write_ctx(|world, _ctx, _reg| {
            world.game_data.set_flag(flag.to_string());
            Ok(())
        });
    }

    fn remove_game_flag(&mut self, flag: &str) {
        with_write_ctx(|world, _ctx, _reg| {
            world.game_data.remove_flag(flag);
            Ok(())
        });
    }

    fn add_stat(&mut self, stat: &str, amount: i64) {
        with_write_ctx(|world, _ctx, reg| {
            let sid = reg
                .get_stat(stat)
                .ok_or_else(|| EffectError::UnknownStat(stat.to_string()))?;
            world.game_data.add_stat(sid, amount as i32);
            Ok(())
        });
    }

    fn set_stat(&mut self, stat: &str, value: i64) {
        with_write_ctx(|world, _ctx, reg| {
            let sid = reg
                .get_stat(stat)
                .ok_or_else(|| EffectError::UnknownStat(stat.to_string()))?;
            world.game_data.set_stat(sid, value as i32);
            Ok(())
        });
    }

    fn set_job_title(&mut self, title: &str) {
        with_write_ctx(|world, _ctx, _reg| {
            world.game_data.job_title = title.to_string();
            Ok(())
        });
    }

    fn add_desire(&mut self, delta: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            world.game_data.add_desire(delta as i32);
            Ok(())
        });
    }

    fn set_desire(&mut self, value: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            world.game_data.set_desire(value as i32);
            Ok(())
        });
    }

    fn advance_time(&mut self, slots: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            for _ in 0..slots {
                world.game_data.advance_time_slot();
            }
            Ok(())
        });
    }

    fn advance_arc(&mut self, arc: &str, to_state: &str) {
        with_write_ctx(|world, _ctx, _reg| {
            world.game_data.advance_arc(arc, to_state);
            Ok(())
        });
    }

    fn fail_red_check(&mut self, skill: &str) {
        with_write_ctx(|world, ctx, _reg| {
            let scene_id = ctx.scene_id.as_deref().unwrap_or("unknown");
            world.game_data.fail_red_check(scene_id, skill);
            Ok(())
        });
    }
}

/// Register the `gd.*` effect mutators. Names are the authored effect vocabulary.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_fn("setGameFlag", Gd::set_game_flag)
        .register_fn("removeGameFlag", Gd::remove_game_flag)
        .register_fn("addStat", Gd::add_stat)
        .register_fn("setStat", Gd::set_stat)
        .register_fn("setJobTitle", Gd::set_job_title)
        .register_fn("addDesire", Gd::add_desire)
        .register_fn("setDesire", Gd::set_desire)
        .register_fn("advanceTime", Gd::advance_time)
        .register_fn("advanceArc", Gd::advance_arc)
        .register_fn("failRedCheck", Gd::fail_red_check);
}

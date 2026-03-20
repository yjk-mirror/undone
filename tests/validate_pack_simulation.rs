#[test]
fn runtime_simulation_reports_reachable_and_unreachable_rows() {
    let result = undone::validate_pack::run_simulation_for_tests(4, 3).expect("simulation");
    let stats = result.stats();
    let landlord = stats
        .iter()
        .find(|stat| stat.scene_id == "base::workplace_landlord")
        .expect("simulation should include workplace_landlord");
    assert!(
        landlord.warning.as_deref() != Some("NEVER FIRES"),
        "workplace_landlord should be reachable in runtime-driven simulation: {:?}",
        landlord.warning
    );

    let campus = stats
        .iter()
        .find(|stat| stat.scene_id == "base::campus_arrival")
        .expect("simulation should keep zero-count rows for unreachable scenes");
    assert!(
        campus.warning.as_deref() == Some("NEVER FIRES"),
        "campus_arrival should remain a zero-count unreachable row in the Robin horizon: {:?}",
        campus.warning
    );
}

/// Verify the simulator's time cadence matches real gameplay:
/// - Opening-arc scenes (consumes_time=false) do NOT advance the clock
/// - free_time/work scenes (consumes_time=true) DO advance the clock
/// - The Robin route naturally reaches week 2+ via scene execution alone
/// - Week-2-gated trigger scenes fire without dev time-travel
#[test]
fn simulator_robin_route_reaches_week_two_naturally() {
    // Run 5 simulations over 4 weeks — enough for reliable coverage.
    let result = undone::validate_pack::run_simulation_for_tests(4, 5).expect("simulation");
    let counts = &result.scene_counts;

    // ── Week-2-gated trigger scenes must fire ──────────────────────────
    // coffee_shop requires `gd.week() >= 2` as a trigger condition.
    // If the simulator never advances time past week 1, this stays at 0.
    let coffee_shop = counts.get("base::coffee_shop").copied().unwrap_or(0);
    assert!(
        coffee_shop > 0,
        "coffee_shop (week >= 2 trigger) should fire at least once across 5 runs, \
         proving the simulator naturally reaches week 2. Got count=0. \
         This means consumes_time slots are not advancing the clock."
    );

    // plan_your_day also requires `gd.week() >= 2` as a trigger.
    let plan_your_day = counts.get("base::plan_your_day").copied().unwrap_or(0);
    assert!(
        plan_your_day > 0,
        "plan_your_day (week >= 2 trigger) should fire at least once across 5 runs. \
         Got count=0."
    );

    // ── Opening-arc scenes should fire exactly once per run ────────────
    // These are once_only triggers in the workplace_opening slot
    // (consumes_time=false). They should fire in every run.
    let arrival = counts.get("base::workplace_arrival").copied().unwrap_or(0);
    assert_eq!(
        arrival, result.runs as u64,
        "workplace_arrival (once_only opening arc) should fire exactly once per run"
    );

    let landlord = counts.get("base::workplace_landlord").copied().unwrap_or(0);
    assert_eq!(
        landlord, result.runs as u64,
        "workplace_landlord (once_only opening arc) should fire exactly once per run"
    );

    // ── Week-1 weighted scenes should fire multiple times ──────────────
    // These have `gd.week() >= 1` conditions and positive weights, so
    // after the opening arc settles they should appear regularly.
    let week1_weighted = [
        "base::park_walk",
        "base::bookstore",
        "base::grocery_store",
        "base::evening_home",
    ];
    for scene_id in &week1_weighted {
        let count = counts.get(*scene_id).copied().unwrap_or(0);
        assert!(
            count > 0,
            "{scene_id} (week >= 1 weighted) should fire at least once across 5 runs of 4 weeks. \
             Got count=0."
        );
    }

    // ── Settled-state work scenes should fire ──────────────────────────
    // These require arcState == 'settled' and consumes_time=true (work slot).
    // If the opening arc never reaches 'settled', these stay at 0.
    let work_scenes = [
        "base::work_standup",
        "base::work_lunch",
        "base::work_corridor",
    ];
    for scene_id in &work_scenes {
        let count = counts.get(*scene_id).copied().unwrap_or(0);
        assert!(
            count > 0,
            "{scene_id} (settled work scene) should fire at least once across 5 runs of 4 weeks. \
             Got count=0. This means the opening arc never reached 'settled' state."
        );
    }

    // ── Campus scenes should never fire (Robin is ROUTE_WORKPLACE) ─────
    let campus_arrival = counts.get("base::campus_arrival").copied().unwrap_or(0);
    assert_eq!(
        campus_arrival, 0,
        "campus_arrival should never fire in a Robin (ROUTE_WORKPLACE) simulation"
    );
}

//! Story-map: derive the base pack's scene-connectivity graph and reconcile it
//! against an authored roadmap, for writers deciding what to write next.
//!
//! Flags and `ARC=STATE` pairs are both treated as **signals**. A scene
//! *produces* signals (effects) and *requires* signals (gates). Dangling =
//! produced but never required (an open door); broken = required but never
//! produced (an unreachable gate).

use serde::Serialize;

/// The full reconciled map. Serializes to the JSON sidecar.
#[derive(Debug, Clone, Serialize, Default)]
pub struct StoryMap {
    pub threads: Vec<Thread>,
    /// Existing scenes claimed by no thread.
    pub orphans: Vec<String>,
    pub drift: Vec<Drift>,
    pub write_next: Vec<WriteNext>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Thread {
    pub name: String,
    pub note: String,
    /// Scenes ordered by signal-dependency (a scene requiring X sorts after the
    /// scene that produces X), id-stable otherwise.
    pub scenes: Vec<SceneNode>,
    /// Signals produced inside this thread that no gate anywhere consumes.
    pub dangling: Vec<DanglingSignal>,
    /// Gate signals required inside this thread that no scene produces.
    pub broken: Vec<BrokenGate>,
    /// Roadmap `planned` scene ids that do not yet exist as files.
    pub planned: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneNode {
    /// Short id (no `pack::` prefix).
    pub id: String,
    pub produces: Vec<String>,
    pub requires: Vec<String>,
    pub status: SceneStatus,
    pub binding: Option<Binding>,
    pub repeatable: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneStatus {
    /// Bound (or an entry/goto target) and all gate signals are producible.
    Reachable,
    /// Bound but a gate signal is produced by nothing.
    BrokenGate,
    /// No schedule binding, not an entry scene, no inbound goto.
    Unbound,
}

#[derive(Debug, Clone, Serialize)]
pub struct Binding {
    pub slot: String,
    pub weight: u32,
    pub once_only: bool,
    pub npc_role: Option<String>,
    pub desire_scaled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DanglingSignal {
    pub signal: String,
    pub set_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrokenGate {
    pub scene: String,
    pub missing: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Drift {
    /// `planned_now_exists` | `empty_prefix`.
    pub kind: String,
    pub thread: String,
    pub detail: String,
}

// Scope note (deliberate): the design lists a fourth write-next category, "thin
// endings" (a thread's terminal scene sets a flag with no consumer). That is
// *exactly* a `dangling` signal already — implementing it as a separate kind
// would double-report the same finding. It is intentionally folded into
// `dangling`; there is no separate `thin_ending` kind. Do not add one without
// first de-duplicating.
#[derive(Debug, Clone, Serialize)]
pub struct WriteNext {
    pub priority: u8,
    /// `dangling` | `broken` | `planned`.
    pub kind: String,
    pub thread: String,
    pub detail: String,
}

/// Build the reconciled story map from a packs directory. Stub for now.
pub fn build_story_map(_packs_dir: &std::path::Path) -> Result<StoryMap, String> {
    Ok(StoryMap::default())
}

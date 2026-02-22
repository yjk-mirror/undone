pub mod game_data;
pub use game_data::GameData;

use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use undone_domain::{FemaleNpc, FemaleNpcKey, MaleNpc, MaleNpcKey, Player};

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub player: Player,
    pub male_npcs: SlotMap<MaleNpcKey, MaleNpc>,
    pub female_npcs: SlotMap<FemaleNpcKey, FemaleNpc>,
    pub game_data: GameData,
}

impl World {
    pub fn male_npc(&self, key: MaleNpcKey) -> Option<&MaleNpc> {
        self.male_npcs.get(key)
    }

    pub fn male_npc_mut(&mut self, key: MaleNpcKey) -> Option<&mut MaleNpc> {
        self.male_npcs.get_mut(key)
    }

    pub fn female_npc(&self, key: FemaleNpcKey) -> Option<&FemaleNpc> {
        self.female_npcs.get(key)
    }

    pub fn female_npc_mut(&mut self, key: FemaleNpcKey) -> Option<&mut FemaleNpc> {
        self.female_npcs.get_mut(key)
    }
}

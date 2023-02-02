use glam::{Vec2, UVec2};

use crate::mapping::*;

#[derive(Debug, Clone, Copy)]
pub struct LedStrip {
    pub length: LedIndex,
}

impl LedStrip {
    pub fn new(length: LedIndex) -> Self {
        Self { length }
    }
}

impl Default for LedStrip {
    fn default() -> Self {
        Self {
            length: 16
        }
    }
}

//todo: probably separate these concerns
impl LedMappingTrait for LedStrip {
    fn get_pos(&self, index: LedIndex) -> UPos {
        let mut x = index % self.length;

        [x as u32, 0].into()
    }

    fn get_size(&self) -> UVec2 {
        UVec2::new(self.length as u32, self.length as u32)
    }

    fn get_num_pixels(&self) -> usize {
        (self.length * self.length) as usize
    }
}

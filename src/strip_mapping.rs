use glam::{Vec2, UVec2};

use crate::mapping::*;

#[derive(Debug, Clone, Copy)]
pub struct StripMapping {
    length: LedIndex,
    inverted: bool,
}

impl StripMapping {
    pub fn new(length: LedIndex, inverted: bool) -> Self {
        Self { length, inverted }
    }
}

impl Default for StripMapping {
    fn default() -> Self {
        Self {
            length: 16,
            inverted: false
        }
    }
}

//todo: probably separate these concerns
impl LedMappingTrait for StripMapping {
    fn get_pos(&self, index: LedIndex) -> UPos {
        let mut x = index % self.length;

        x = if self.inverted {
            self.length - 1 - x
        } else {
            x
        };

        [x as u32, 0].into()
    }

    fn get_size(&self) -> UVec2 {
        UVec2::new(self.length as u32, 1)
    }

    fn get_num_pixels(&self) -> usize {
        self.length as usize
    }
}

use glam::{Vec2, UVec2};

use crate::mapping::*;

#[derive(Debug, Clone, Copy)]
pub struct LedMatrix {
    pub width: LedIndex,
}

impl LedMatrix {
    pub fn new(width: LedIndex) -> Self {
        Self { width }
    }
}

impl Default for LedMatrix {
    fn default() -> Self {
        Self {
            width: 16
        }
    }
}

//todo: probably separate these concerns
impl LedMappingTrait for LedMatrix {
    fn get_pos(&self, index: LedIndex) -> UPos {
        let mut x = index % self.width;
        let y = index / self.width;

        let is_even_row = y % 2 == 1;

        if is_even_row {
            x = self.width - 1 - x;
        }

        [y as u32, x as u32].into()
    }

    fn get_size(&self) -> UVec2 {
        UVec2::new(self.width as u32, self.width as u32)
    }

    fn get_num_pixels(&self) -> usize {
        (self.width * self.width) as usize
    }
}

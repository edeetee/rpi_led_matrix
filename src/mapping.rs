use enum_dispatch::enum_dispatch;
use glam::{UVec2};

use crate::matrix_mapping::LedMatrix;

/** Index of a pixel inside a given fixture.
 * Each pixel is made up of 3 dmx channels
 * The ordering of the Index is the same order as the DMX channels
 * Between 0 < num_pixels
 */
pub type LedIndex = usize;

/**
 * Position of the pixel in 2d space, starting at 0,0
 */
pub type UPos = UVec2;

pub const CHANNELS_PER_UNIVERSE: usize = 510;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DmxAddress {
    /// Pretty much the universe
    pub universe: u8,
    /// The DMX512 address
    pub channel: usize,
}

impl DmxAddress {
    ///Calulate the dmx address for a given pixel
    pub fn pixel_offset(&self, index: LedIndex) -> DmxAddress {
        let rgb_index = index * 3;

        let absolute_index = rgb_index + self.channel as LedIndex;

        //split the absolute channel into dmx channels and universes
        let dmx_channel = absolute_index % CHANNELS_PER_UNIVERSE as LedIndex;
        let dmx_universe =
            self.universe + (absolute_index / CHANNELS_PER_UNIVERSE as LedIndex) as u8;

        DmxAddress {
            channel: dmx_channel,
            universe: dmx_universe,
        }
    }
}

impl From<(usize, u8)> for DmxAddress {
    fn from((channel, universe): (usize, u8)) -> Self {
        Self { channel, universe }
    }
}

#[enum_dispatch(LedMappingEnum)]
/// Maps an led fixture to 2d coordinates
pub trait LedMappingTrait: Clone {
    /// Get the position of the pixel in 2d space
    fn get_pos(&self, index: LedIndex) -> UPos;

    //max size of the whole fixture
    fn get_size(&self) -> UVec2;

    //number of pixels in the fixture
    fn get_num_pixels(&self) -> usize;
}

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum LedMappingEnum {
    LedMatrix
}
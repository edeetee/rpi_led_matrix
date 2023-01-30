use glam::{UVec2};

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
    /**
     * Pretty much the universe
     */
    pub universe: u8,
    /**
     * The DMX address
     */
    pub channel: usize,
}

impl From<(usize, u8)> for DmxAddress {
    fn from((channel, universe): (usize, u8)) -> Self {
        Self { channel, universe }
    }
}

/**
 * All led mappings are continuous blocks of dmx channels.
 */
pub trait LedMapping {
    /**
     * Get the position of the pixel in 2d space
     */
    fn get_pos(&self, index: LedIndex) -> UPos;
    /**
     * Get the DMX index and universe of a given pixel
     */
    fn get_dmx_mapping(&self, index: LedIndex) -> DmxAddress;

    fn get_num_pixels(&self) -> usize;

    fn generate_empty_data(&self) -> Vec<[u8; 3]> {
        vec![[0, 0, 0]; self.get_num_pixels()]
    }
}
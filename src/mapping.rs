use artnet_protocol::PortAddress;
use glam::UVec2;

/** Index of a pixel inside a given fixture.
 * Each pixel is made up of 3 dmx channels
 * The ordering of the Index is the same order as the DMX channels
 * Between 0 < num_pixels
 */
pub type LedIndex = usize;

/**
 * Position of the pixel in 2d space
 */
pub type Pos = UVec2;

#[derive(Debug)]
pub struct DmxAddress {
    /**
     * The DMX address
     */
    pub channel: usize,
    /**
     * Pretty much the universe
     */
    pub universe: u8
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
    fn get_pos(&self, index: LedIndex) -> Pos;
    /**
     * Get the DMX index and universe of a given pixel
     */
    fn get_dmx_mapping(&self, index: LedIndex) -> DmxAddress;

    fn get_num_pixels(&self) -> usize;

    fn generate_empty_data(&self) -> Vec<[u8;3]> {
        vec![[0,0,0]; self.get_num_pixels()]
    }
}
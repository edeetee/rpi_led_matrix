use crate::mapping::*;


#[derive(Debug)]
pub struct LedMatrix {
    pub address: DmxAddress,
    pub width: LedIndex,
}

impl LedMatrix {
    pub fn new(width: LedIndex, address: DmxAddress) -> Self {
        Self {
            width,
            address
        }
    }
}

impl Default for LedMatrix {
    fn default() -> Self {
        Self { width: 16, address: 
            DmxAddress {
                channel: 0,
                universe: 0
            } 
        }
    }
}


const CHANNELS_PER_UNIVERSE: usize = 510;

//todo: probably separate these concerns
impl LedMapping for LedMatrix {
    /**
     * 
     */
    fn get_pos(&self, index: LedIndex) -> Pos {
        let mut x = index % self.width;
        let y = index / self.width;

        let is_even_row = y % 2 == 1;

        if is_even_row {
            x = self.width - 1 - x;
        }

        [x as u32, y as u32].into()
    }

    fn get_dmx_mapping(&self, index: LedIndex) -> DmxAddress{
        // let pos = self.get_pos(index);

        // let index = pos.y*self.width + pos.x;
        let rgb_index = index*3;

        let absolute_index = rgb_index + self.address.channel as LedIndex;

        // absolute_index

        //split the absolute channel into dmx channels and universes
        let dmx_channel = absolute_index % CHANNELS_PER_UNIVERSE as LedIndex;
        let dmx_universe = self.address.universe + (absolute_index / CHANNELS_PER_UNIVERSE as LedIndex) as u8;

        DmxAddress {
            channel: dmx_channel,
            universe: dmx_universe
        }
    }

    fn get_num_pixels(&self) -> usize {
        (self.width*self.width) as usize
    }

    
}
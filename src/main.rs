use std::{net::UdpSocket, thread::sleep, time::{Duration, Instant}, collections::HashMap};
use artnet_protocol::{PortAddress, Output, ArtCommand};
use glam::Vec2;
use mapping::{LedMapping};
use matrix_mapping::LedMatrix;

use draw::{Context};
use crate::draw::draw;

mod mapping;
mod matrix_mapping;
mod draw;

const PORT: u16 = 6454;
const BIND_ADDR: &'static str = "192.254.250.10";
const ARTNET_ADDR: &str = &"192.254.250.9";

#[derive(Debug)]
struct LedMatrixData {
    mapping: LedMatrix,
    // data: Vec<[u8; 3]>,
    pos_offset: Vec2
}

impl LedMatrixData {
    fn new(mapping: LedMatrix, pos_offset: Vec2) -> LedMatrixData {
        LedMatrixData {
            // data: mapping.generate_empty_data(),
            mapping,
            pos_offset
        }

    }
}

fn main() {
    let socket = UdpSocket::bind((BIND_ADDR, 0)).expect("Failed to bind to UDP socket");
    let addr = (ARTNET_ADDR, PORT);
    socket.connect(addr).expect("Failed to connect to the artnet server");

    // let led_mapping = LedSquare::new(16, (0, 0).into());
    // let mut pixel_data = vec![[0,0,0]; led_mapping.get_num_pixels()];

    let strip1 = LedMatrixData::new(LedMatrix::new(16, (0, 0).into()), Vec2::ZERO);
    let addr_after_strip1 = strip1.mapping.get_dmx_mapping(strip1.mapping.get_num_pixels());
    let strip2 = LedMatrixData::new(LedMatrix::new(16, addr_after_strip1), Vec2::new(-16.0, 0.0));

    let squares: Vec<LedMatrixData> = vec![
        strip1, strip2
    ];

    let mut dmx_data: HashMap<PortAddress, Vec<u8>> = Default::default();

    let start_time = Instant::now();

    println!("Rendering to: {squares:?}");

    loop {
        // pixels.push((0,0,0));
        let elapsed = start_time.elapsed();
        let elapsed_seconds = elapsed.as_secs_f32();

        let ctx = Context {
            elapsed_seconds,
            elapsed
        };

        for fixture in &squares {
            let mapping = &fixture.mapping;
            // let fixture_offset = fixt

            for i in 0..mapping.get_num_pixels() {

                let dmx_target = mapping.get_dmx_mapping(i);
                let dmx_channel_start = dmx_target.channel;
    
                let output = dmx_data.entry(dmx_target.universe.into())
                    .or_insert(vec![]);
    
                //expand dmx channel to match pixels
                while output.len() <= dmx_channel_start as usize +2 {
                    output.push(0)
                }

                let pos_i = mapping.get_pos(i);
                let pos_f = pos_i.as_vec2();
            
                let center_offset = -Vec2::new(mapping.width as f32/2.0-0.5, mapping.width as f32/2.0-0.5);
    
                let color = draw(pos_f+center_offset+fixture.pos_offset, &ctx);
                // let output = color
    
                for (dmx_chan_offset, color) in [color.red, color.green, color.blue].iter().enumerate() {
                    output[dmx_channel_start as usize + dmx_chan_offset] = *color;
                }
            }
        }

        //todo: don't clone here
        for (port_address, data) in &dmx_data {
            let command = ArtCommand::Output(Output {
                data: data.clone().into(),
                port_address: *port_address,
                ..Default::default()
            });

            // println!("SENDING: {command:?}");

            socket.send(&command.write_to_buffer().unwrap()).unwrap();
        }
        

        
        sleep(Duration::from_secs_f32(1.0/240.0));
    }
}


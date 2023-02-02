use artnet_protocol::{ArtCommand, Output, PortAddress};
use clap::Parser;

use glam::Vec2;
use mapping::{DmxAddress, LedMappingTrait, LedMappingEnum};
use matrix_mapping::MatrixMapping;
use spin_sleep::{SpinSleeper, sleep};

use std::{
    collections::HashMap,
    fmt::Debug,
    net::UdpSocket,
    sync::{mpsc::{sync_channel}},
    thread::{self},
    time::{Duration, Instant},
};
use draw::DrawContext;

mod draw;
mod mapping;
mod matrix_mapping;
mod strip_mapping;
mod cli;

use crate::{draw::draw, strip_mapping::StripMapping};

#[cfg(feature = "gui")]
mod previs_ui;

#[cfg(feature = "jack")]
mod audio;

const PORT: u16 = 6454;
const BIND_ADDR: &'static str = "192.168.11.5";
const ARTNET_ADDR: &str = &"192.168.11.4";

#[derive(Debug, Clone)]
pub struct LedMappingInfo {
    mapping: LedMappingEnum,
    dmx_address: DmxAddress,
    pos_offset: Vec2,
}

impl LedMappingInfo {
    fn new(mapping: LedMappingEnum, pos_offset: Vec2, dmx_address: DmxAddress) -> Self {
        LedMappingInfo {
            mapping,
            pos_offset,
            dmx_address
        }
    }
}

///Led data structured in the dmx alignment
#[derive(Clone)]
pub struct LedData {
    info: LedMappingInfo,
    data: Vec<[u8; 3]>
}

#[derive(Debug)]
pub struct LedFrameInfo {
    target_period: Duration,
    last_period: Duration,
    rendering_period: Duration,
}

fn chained_led_mappings<'a, T: LedMappingTrait>(address: DmxAddress, make_mapping: impl Fn() -> T) -> impl Iterator<Item = (DmxAddress, T)> {
    let first_strip = (address, make_mapping());

    std::iter::successors(Some(first_strip), move |(addr, mapping)| {
        let next_address = addr.pixel_offset(mapping.get_num_pixels());
        Some((next_address, make_mapping()))
    })
}

fn chained_led_matrices(width: usize, address: DmxAddress, pos: impl IntoIterator<Item=Vec2>) -> impl Iterator<Item=LedMappingInfo> {
    chained_led_mappings(address, move || MatrixMapping::new(width))
        .zip(pos)
        .map(|((address, matrix), pos)| LedMappingInfo::new(matrix.into(), pos, address))
}

fn render_leds(ctx: DrawContext, matrices: &[LedMappingInfo], dmx_data: &mut HashMap<PortAddress, [u8; 512]>) -> Vec<LedData> {
    let mut led_data: Vec<LedData> = Vec::with_capacity(matrices.len());
    
    for fixture in matrices {
        let mapping = &fixture.mapping;

        let mut pixels = vec![[0,0,0]; mapping.get_num_pixels()];

        for i in 0..mapping.get_num_pixels() {
            let dmx_target = fixture.dmx_address.pixel_offset(i);
            let dmx_channel_start = dmx_target.channel;

            let dmx_universe_output = dmx_data
                .entry(dmx_target.universe.into())
                .or_insert([0; 512]);

            let pos_i = mapping.get_pos(i);
            let pos_f = pos_i.as_vec2();

            let center_offset = -glam::Vec2::new(0.5, 0.5)*16.0 + glam::Vec2::new(0.5,0.5);

            let pos_f_scale = match mapping {
                LedMappingEnum::MatrixMapping(_) => Vec2::ONE,
                LedMappingEnum::StripMapping(_) => Vec2::ONE*0.1,
            };

            let draw_pos = pos_f*pos_f_scale + center_offset + fixture.pos_offset;

            let color = draw(&ctx, draw_pos);

            pixels[i] = [color.red, color.green, color.blue];

            dmx_universe_output[dmx_channel_start..dmx_channel_start+3]
                .swap_with_slice(&mut [color.red, color.green, color.blue]);
        }

        led_data.push(LedData { info: fixture.clone(), data: pixels });
    }
    
    led_data
}

// fn mapping_once(dmx_address: DmxAddress, pos_offset: Vec2, mapping: LedMappingEnum)-> std::iter::Once<LedMappingInfo> {
//     std::iter::once(LedMappingInfo { 
//         mapping, 
//         dmx_address, 
//         pos_offset
//     })
// }

fn main() {
    let args = cli::Args::parse();

    let socket = UdpSocket::bind((BIND_ADDR, 0));
    let addr = (ARTNET_ADDR, PORT);

    match &socket {
        Ok(socket) => {
            socket
                .connect(addr)
                .expect("Failed to connect to the artnet server");
        }
        Err(err) => {
            eprintln!("Could not bind to socket. \n{err:?}");
            if ! cfg!(feature = "gui"){
                panic!();
            }
            eprintln!("Continuing with UI.");
        },
    }

    let matrices = 
        chained_led_matrices(16, (0,44).into(), vec![Vec2::new(-8.0, 0.0), Vec2::new(8.0, 0.0)])
        .chain(
            chained_led_matrices(16, (0,40).into(), vec![Vec2::new(-8.0, 16.0), Vec2::new(8.0, 16.0)])
        )
        .chain(
            chained_led_matrices(16, (0,48).into(), vec![Vec2::new(0.0, 2.0*16.0), Vec2::new(0.0, 3.0*16.0)])
        )
        
        .chain(std::iter::once(LedMappingInfo { 
            mapping: StripMapping::new(6, false).into(), 
            dmx_address: (0,36).into(), 
            pos_offset: Vec2::new(16.0, 7.5)
        }))
        
        .chain(std::iter::once(LedMappingInfo { 
            mapping: StripMapping::new(100, true).into(), 
            dmx_address: (0,38).into(), 
            pos_offset: Vec2::new(8.0, 7.5)
        }))

        .chain(std::iter::once(LedMappingInfo { 
            mapping: StripMapping::new(6, false).into(), 
            dmx_address: (0,34).into(), 
            pos_offset: Vec2::new(16.0, 7.5)
        }))
        
        .chain(std::iter::once(LedMappingInfo { 
            mapping: StripMapping::new(100, true).into(), 
            dmx_address: (0,32).into(), 
            pos_offset: Vec2::new(8.0, 7.5)
        }))

        .collect::<Vec<_>>();

    let matrices_clone = matrices.clone();

    println!("DMX Squares: {matrices:#?}");
    
    #[cfg(feature = "jack")]
    let audio_rx = audio::get_audio();
    let mut noise = noise::Perlin::default();
 
    let (led_frame_tx, led_frame_rx) = sync_channel(1);
    let (led_frame_info_tx, led_frame_data_rx) = sync_channel(1);

    let dmx_thread = thread::spawn(move || {

        let start_time = Instant::now();

        let process_led_frame = || {
            let mut dmx_data: HashMap<PortAddress, [u8; 512]> = Default::default();

            let elapsed = start_time.elapsed();
            let elapsed_seconds = elapsed.as_secs_f32();
            
            let ctx = DrawContext {
                elapsed_seconds,
                elapsed,
                noise,

                #[cfg(feature = "jack")]
                audio: audio_rx.recv().unwrap(),
                #[cfg(not(feature = "jack"))]
                audio: vec![0.0]
            };

            let led_data = render_leds(ctx, &matrices, &mut dmx_data);

            led_frame_tx.try_send(led_data).ok();

            for (port_address, data) in &dmx_data {
                let command = ArtCommand::Output(Output {
                    data: data.to_vec().into(),
                    port_address: *port_address,
                    ..Default::default()
                });

                match socket {
                    Ok(ref socket_actual) => {
                        match socket_actual.send(&command.write_to_buffer().unwrap()) {
                            Ok(_) => {},
                            Err(err) => {eprintln!("Failed to send via socket {err:?}. Continuing..")},
                        }
                    }
                    Err(_) => {}
                }
            }
        };

        let target_loop_period = Duration::from_millis(1000 / 60);
        let mut last_start_frame_time = Instant::now();

        let sleeper = SpinSleeper::new(10_000_000);

        loop {
            let elapsed_frame_time = last_start_frame_time.elapsed();
            last_start_frame_time = Instant::now();
            
            process_led_frame();

            match led_frame_info_tx.try_send(LedFrameInfo {
                            target_period: target_loop_period,
                            last_period: elapsed_frame_time,
                            rendering_period: last_start_frame_time.elapsed()
                        }) {
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    panic!("Led data receiver disconnected!");
                }
                _ => {},
            };

            sleeper.sleep(target_loop_period.saturating_sub(last_start_frame_time.elapsed()));
        }
    });


    if !args.headless && cfg!(feature = "gui") {
        #[cfg(feature = "gui")]
        previs_ui::run_gui(matrices_clone, led_frame_rx, led_frame_data_rx);
    } else {
        loop {
            let data = led_frame_data_rx.recv().unwrap();

            println!("{data:?}");
            sleep(Duration::from_millis(1000));
        }
    }

    dmx_thread.join().unwrap();
}


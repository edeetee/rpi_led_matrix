use artnet_protocol::{ArtCommand, Output, PortAddress};
use clap::Parser;

#[cfg(feature = "gui")]
use egui::{Color32, ColorImage, TextureHandle, TextureOptions};

use glam::Vec2;
use mapping::{DmxAddress, LedMapping};
use matrix_mapping::LedMatrix;
use spin_sleep::{SpinSleeper, sleep};

use std::{
    collections::HashMap,
    fmt::Debug,
    net::UdpSocket,
    sync::{mpsc::{channel, sync_channel, Sender, SyncSender, Receiver}},
    thread::{self},
    time::{Duration, Instant},
};
use draw::Context;

mod draw;
mod mapping;
mod matrix_mapping;
mod cli;

use crate::{draw::draw};

#[cfg(feature = "gui")]
mod previs_ui;
#[cfg(feature = "gui")]
use crate::previs_ui::PrevisApp;

#[cfg(feature = "jack")]
mod audio;

const PORT: u16 = 6454;
const BIND_ADDR: &'static str = "192.168.11.5";
const ARTNET_ADDR: &str = &"192.168.11.4";

#[derive(Debug, Clone)]
pub struct LedMatrixInfo {
    mapping: LedMatrix,
    pos_offset: Vec2,
}

impl LedMatrixInfo {
    fn new(mapping: LedMatrix, pos_offset: Vec2) -> LedMatrixInfo {
        LedMatrixInfo {
            mapping,
            pos_offset,
        }
    }
}

///Led data structured in the dmx alignment
#[derive(Clone)]
pub struct LedMatrixData {
    info: LedMatrixInfo,
    data: Vec<[u8; 3]>
}

impl LedMatrixData {
    fn new(info: LedMatrixInfo, data: Vec<[u8; 3]>) -> Self {
        Self{
            info,
            data
        }
    }
}

#[derive(Debug)]
pub struct LedFrameInfo {
    target_period: Duration,
    last_period: Duration,
    rendering_period: Duration,
}

fn chained_led_matrices(width: usize, address: DmxAddress) -> impl Iterator<Item = LedMatrix> {
    let first_strip = LedMatrix::new(width, address);

    std::iter::successors(Some(first_strip), |prev_strip| {
        let next_address = prev_strip.get_dmx_mapping(prev_strip.get_num_pixels());
        Some(LedMatrix::new(16, next_address))
    })
}

fn pos_to_led_info(width: usize, address: DmxAddress, pos: impl IntoIterator<Item=Vec2>) -> impl Iterator<Item=LedMatrixInfo> {
    chained_led_matrices(width, address)
        .zip(pos)
        .map(|(matrix, pos)| LedMatrixInfo::new(matrix, pos))
}

fn draw_leds(ctx: Context, matrices: &[LedMatrixInfo], dmx_data: &mut HashMap<PortAddress, [u8; 512]>) -> Vec<LedMatrixData> {
    let mut led_data: Vec<LedMatrixData> = Vec::with_capacity(matrices.len());
    
    for fixture in matrices {
        let mapping = &fixture.mapping;

        let mut pixels = vec![[0,0,0]; mapping.get_num_pixels()];

        for i in 0..mapping.get_num_pixels() {
            let dmx_target = mapping.get_dmx_mapping(i);
            let dmx_channel_start = dmx_target.channel;

            let dmx_universe_output = dmx_data
                .entry(dmx_target.universe.into())
                .or_insert([0; 512]);

            let pos_i = mapping.get_pos(i);
            let pos_f = pos_i.as_vec2();

            let center_offset = -glam::Vec2::new(0.5, 0.5)*16.0 + glam::Vec2::new(0.5,0.5);

            let draw_pos = pos_f + center_offset + fixture.pos_offset;

            let color = draw(draw_pos, &ctx);

            pixels[i] = [color.red, color.green, color.blue];

            dmx_universe_output[dmx_channel_start..dmx_channel_start+3]
                .swap_with_slice(&mut [color.red, color.green, color.blue]);
        }

        led_data.push(LedMatrixData { info: fixture.clone(), data: pixels });
    }
    
    led_data
}

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
        Err(err) => eprintln!("Could not bind to socket. \n{err:?}\n Continuing with UI."),
    }


    let matrices = pos_to_led_info(16, (0,44).into(), 
    vec![Vec2::new(-8.0, 0.0), Vec2::new(8.0, 0.0)])
        .chain(
            pos_to_led_info(16, (0,40).into(), vec![Vec2::new(-8.0, 16.0), Vec2::new(8.0, 16.0)])
        )
        .chain(
            pos_to_led_info(16, (0,48).into(), vec![Vec2::new(0.0, 2.0*16.0), Vec2::new(0.0, 3.0*16.0)])
        )
        .collect::<Vec<_>>();

    let matrices_clone = matrices.clone();

    println!("DMX Squares: {matrices:#?}");
    
    #[cfg(feature = "jack")]
    let audio_rx = audio::get_audio();
 
    let (led_frame_tx, led_frame_rx) = sync_channel(1);
    let (led_frame_info_tx, led_frame_data_rx) = sync_channel(1);

    let dmx_thread = thread::spawn(move || {

        let start_time = Instant::now();

        let process_led_frame = || {
            let mut dmx_data: HashMap<PortAddress, [u8; 512]> = Default::default();

            let elapsed = start_time.elapsed();
            let elapsed_seconds = elapsed.as_secs_f32();
            
            let ctx = Context {
                elapsed_seconds,
                elapsed,

                #[cfg(feature = "jack")]
                audio: audio_rx.recv().unwrap(),
                #[cfg(not(feature = "jack"))]
                audio: vec![0.0]
            };

            let led_data = draw_leds(ctx, &matrices, &mut dmx_data);

            led_frame_tx.try_send(led_data);

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


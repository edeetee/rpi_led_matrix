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
    sync::{mpsc::{channel, sync_channel}},
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
#[cfg(feature = "jack")]
use crate::audio::get_audio;



const PORT: u16 = 6454;
const BIND_ADDR: &'static str = "192.168.6.5";
const ARTNET_ADDR: &str = &"192.168.6.4";

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

#[derive(Debug)]
pub struct LedFrameData {
    target_period: Duration,
    last_period: Duration
}

#[cfg(feature = "gui")]
fn black_square_image(width: usize) -> ColorImage {
    ColorImage::new([width, width], Color32::WHITE)
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

fn draw_leds(ctx: Context, #[cfg(feature = "gui")] previs_textures: &mut [TextureHandle], matrices: &[LedMatrixInfo], dmx_data: &mut HashMap<PortAddress, [u8; 512]>) {
    for (i, fixture) in matrices.iter().enumerate() {
        let mapping = &fixture.mapping;
        // let fixture_offset = fixt
        #[cfg(feature = "gui")]
        let mut previs_image = black_square_image(mapping.width);

        for i in 0..mapping.get_num_pixels() {
            let dmx_target = mapping.get_dmx_mapping(i);
            let dmx_channel_start = dmx_target.channel;

            let dmx_universe_output = dmx_data
                .entry(dmx_target.universe.into())
                .or_insert([0; 512]);

            let pos_i = mapping.get_pos(i);
            let pos_f = pos_i.as_vec2();

            let center_offset = -glam::Vec2::new(0.5, 1.0)*16.0 + glam::Vec2::new(0.5,0.5);

            let draw_pos = pos_f + center_offset + fixture.pos_offset;

            let color = draw(draw_pos, &ctx);


            #[cfg(feature = "gui")]
            {
                let pixel_index: usize = pos_i.x as usize + pos_i.y as usize * previs_image.width();
                previs_image.pixels[pixel_index] = Color32::from_rgb(color.red, color.green, color.blue);
            }
            
            dmx_universe_output[dmx_channel_start..dmx_channel_start+3]
                .swap_with_slice(&mut [color.red, color.green, color.blue]);
        }

        #[cfg(feature = "gui")]
        if i < previs_textures.len() {
            let texture_handle = &mut previs_textures[i];
            texture_handle.set(previs_image, NEAREST_IMG_FILTER);
        }
    }
}

fn main() {
    let args = cli::Args::parse();


    let socket = UdpSocket::bind((BIND_ADDR, 0)).ok();
    let addr = (ARTNET_ADDR, PORT);

    if let Some(ref socket) = socket {
        socket
            .connect(addr)
            .expect("Failed to connect to the artnet server");
    } else {
        eprintln!("Could not connect to socket. Continuing with UI.")
    }

    // let matrix_positions = vec![
    //     Vec2::new(-8.0, 0.0),
    //     Vec2::new(8.0, 0.0),
    //     Vec2::new(-8.0, 16.0),
    //     Vec2::new(8.0, 16.0),
    //     Vec2::new(0.0, 2.0 * 16.0),
    //     Vec2::new(0.0, 3.0 * 16.0),
    //     // Vec2::new()
    // ];

    // let top_row = chained_led_matrices(16, (0,0).into())
    //     .zip(vec![
    //         Vec2::new(-8.0, 0.0),
    //         Vec2::new(8.0, 0.0),
    //     ]);



    // let matrices = chained_led_matrices(16, (0, 0).into())
    //     .zip(matrix_positions)
    //     .map(|(matrix, pos)| LedMatrixInfo::new(matrix, pos))
    //     .collect::<Vec<_>>();

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

    #[cfg(feature = "gui")]
    let (previs_textures_tx, previs_textures_rx) = channel();

    println!("DMX Squares: {matrices:#?}");
    
    #[cfg(feature = "jack")]
    let audio_rx = get_audio();
 
    let (led_frame_data_tx, led_frame_data_rx) = sync_channel(1);

    let dmx_thread = thread::spawn(move || {

        #[cfg(feature = "gui")]
        let mut previs_textures: Vec<TextureHandle> = previs_textures_rx.recv().unwrap_or(vec![]);

        let start_time = Instant::now();

        let mut process_led_frame = move || {
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

            draw_leds(ctx, #[cfg(feature = "gui")] &mut previs_textures, &matrices, &mut dmx_data);

            for (port_address, data) in &dmx_data {
                let command = ArtCommand::Output(Output {
                    data: data.to_vec().into(),
                    port_address: *port_address,
                    ..Default::default()
                });

                match socket {
                    Some(ref socket_actual) => {
                        socket_actual
                            .send(&command.write_to_buffer().unwrap())
                            .unwrap();
                    }
                    None => {}
                }
            }
        };

        let target_loop_period = Duration::from_millis(1000 / 120);
        let mut last_start_frame_time = Instant::now();

        let sleeper = SpinSleeper::new(10_000_000);

        loop {
            let elapsed_frame_time = last_start_frame_time.elapsed();
            last_start_frame_time = Instant::now();

            process_led_frame();

            match led_frame_data_tx.try_send(LedFrameData {
                            target_period: target_loop_period,
                            last_period: elapsed_frame_time
                        }) {
                Ok(_) => {},
                Err(std::sync::mpsc::TrySendError::Full(_)) => {},
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    panic!("Led data receiver disconnected!");
                }
            };

            // let 
            sleeper.sleep(target_loop_period.saturating_sub(last_start_frame_time.elapsed()));
        }
    });

    if !args.headless && cfg!(feature = "gui") {
        #[cfg(feature = "gui")]
        show_ui(matrices_clone, led_frame_data_rx, previs_textures_tx);
    } else {
        loop {
            let data = led_frame_data_rx.recv().unwrap();

            println!("{data:?}");
            sleep(Duration::from_millis(1000));
        }
    }

    dmx_thread.join().unwrap();
}

use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
#[cfg(feature = "gui")]
fn show_ui(matrices: Vec<LedMatrixInfo>, led_frame_data_rx: Receiver<LedFrameData>, previs_textures_tx: Sender<Vec<TextureHandle>>) {

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(340.0, 700.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "LED Previs",
        native_options,
        Box::new(move |cc| {
            let previs_textures = matrices
                .iter()
                .map(|info| {
                    let image = black_square_image(info.mapping.width);

                    cc.egui_ctx
                        .load_texture(format!("img{info:?}"), image, NEAREST_IMG_FILTER)
                })
                .collect::<Vec<_>>();

            let screens = matrices
                .iter()
                .cloned()
                .zip(previs_textures.iter().cloned())
                .collect();

            previs_textures_tx
                .send(previs_textures)
                .expect("Failed to send gui texture handles across threads");

            Box::new(PrevisApp::new(screens, led_frame_data_rx))
        }),
    );
}

#[cfg(feature = "gui")]
const NEAREST_IMG_FILTER: TextureOptions = TextureOptions {
    magnification: egui::TextureFilter::Nearest,
    minification: egui::TextureFilter::Nearest,
};

use std::{net::UdpSocket, thread::{sleep, self}, time::{Duration, Instant}, collections::HashMap, sync::{Arc, Mutex, RwLock}, fmt::Debug};
use artnet_protocol::{PortAddress, Output, ArtCommand};
use egui::{ColorImage, Color32, TextureOptions, TextureHandle};
use glam::Vec2;
use mapping::{LedMapping, DmxAddress};
use matrix_mapping::LedMatrix;

use draw::{Context};
use crate::{draw::draw, previs_ui::{PrevisApp}};

mod mapping;
mod matrix_mapping;
mod draw;
mod previs_ui;

const PORT: u16 = 6454;
const BIND_ADDR: &'static str = "192.254.250.10";
const ARTNET_ADDR: &str = &"192.254.250.9";

#[derive(Debug, Clone)]
pub struct LedMatrixInfo {
    mapping: LedMatrix,
    pos_offset: Vec2
}

impl LedMatrixInfo {
    fn new(mapping: LedMatrix, pos_offset: Vec2) -> LedMatrixInfo {
        LedMatrixInfo {
            mapping,
            pos_offset,
        }
    }
}

fn black_square_image(width: usize)-> ColorImage {
    ColorImage::new([width, width], Color32::WHITE)
}

fn chained_led_matrices(width: usize, address: DmxAddress) -> impl Iterator<Item=LedMatrix> {
    let first_strip = LedMatrix::new(width, address);

    std::iter::successors(Some(first_strip), |prev_strip| {
        let next_address = prev_strip.get_dmx_mapping(prev_strip.get_num_pixels());
        Some(LedMatrix::new(16, next_address))
    })
}

fn main() {
    let socket = UdpSocket::bind((BIND_ADDR, 0)).ok();
    let addr = (ARTNET_ADDR, PORT);

    if let Some(ref socket) = socket {
        socket.connect(addr).expect("Failed to connect to the artnet server");
    } else {
        eprintln!("Could not connect to socket. Continuing with UI.")
    }
    
    let matrix_chain = 
        chained_led_matrices(16, (0,0).into());

    let matrix_positions = vec![
        Vec2::new(-8.0, 0.0),
        Vec2::new(8.0, 0.0),

        Vec2::new(-8.0, 16.0),
        Vec2::new(8.0, 16.0),

        Vec2::new(0.0, 2.0*16.0),

        Vec2::new(0.0, 3.0*16.0),
        // Vec2::new()
    ];
    
    let all_matrices = matrix_chain.zip(matrix_positions)
        .map(|(matrix, pos)| {
            LedMatrixInfo::new(matrix, pos)
        })
        .collect::<Vec<_>>();

    let matrices_clone = all_matrices.clone();

    
    let previs_textures: Arc<RwLock<Option<Vec<TextureHandle>>>> = Default::default();
    let previs_tex_clone = previs_textures.clone();

    println!("DMX Squares: {all_matrices:#?}");

    thread::spawn(move || {
        let mut dmx_data: HashMap<PortAddress, [u8; 512]> = Default::default();
        let start_time = Instant::now();

        loop {
            // let mut previs_textures = previs_textures.write().unwrap().as_mut();

            let elapsed = start_time.elapsed();
            let elapsed_seconds = elapsed.as_secs_f32();
    
            let ctx = Context {
                elapsed_seconds,
                elapsed
            };
    
            for (i, fixture) in all_matrices.iter().enumerate() {
                let mapping = &fixture.mapping;
                // let fixture_offset = fixt
                let mut previs_image = black_square_image(mapping.width);
    
                for i in 0..mapping.get_num_pixels() {
    
                    let dmx_target = mapping.get_dmx_mapping(i);
                    let dmx_channel_start = dmx_target.channel;
        
                    let dmx_universe_output = dmx_data.entry(dmx_target.universe.into())
                        .or_insert([0; 512]);
    
                    let pos_i = mapping.get_pos(i);
                    let pos_f = pos_i.as_vec2();
                
                    let center_offset = -Vec2::new(mapping.width as f32/2.0-0.5, mapping.width as f32/2.0-0.5);
        
                    let draw_pos = pos_f+center_offset+fixture.pos_offset;
        
                    let color = draw(draw_pos, &ctx);

                    let pixel_index: usize = pos_i.x as usize + pos_i.y as usize * mapping.width;
                    previs_image.pixels[pixel_index] = Color32::from_rgb(color.red, color.green, color.blue);
        
                    for (dmx_chan_offset, color) in [color.red, color.green, color.blue].iter().enumerate() {
                        dmx_universe_output[dmx_channel_start as usize + dmx_chan_offset] = *color;
                    }
                }

                if let Some(previs_textures) = previs_textures.write().unwrap().as_mut() {
                    let texture_handle = &mut previs_textures[i];
                    texture_handle.set(previs_image, nearest_img_filter);
                }

                // println!("PIXELS: {:?}", previs_image.pixels);
            }
    
            for (port_address, data) in &dmx_data {
                let command = ArtCommand::Output(Output {
                    data: data.to_vec().into(),
                    port_address: *port_address,
                    ..Default::default()
                });

                match socket {
                    Some(ref socket_actual) => {socket_actual.send(&command.write_to_buffer().unwrap()).unwrap();},
                    None => {},
                }
    
            }
            
            sleep(Duration::from_secs_f32(1.0/240.0));
        }
    });

    let native_options = eframe::NativeOptions::default();
 

    eframe::run_native("LED Previs", native_options, 
        Box::new(move |cc| {

            let previs_textures = matrices_clone.iter()
                .map(|info| {
                    let image = black_square_image(info.mapping.width);
                    let texture_handle = cc.egui_ctx.load_texture(format!("img{info:?}"), image, nearest_img_filter);

                    texture_handle
                }).collect::<Vec<_>>();

            let screens = matrices_clone.iter().cloned()
                .zip(previs_textures.iter().cloned())
                .collect();

            *previs_tex_clone.write().unwrap() = Some(previs_textures);

            Box::new(PrevisApp::new(screens))
        })
    );
}

const nearest_img_filter: TextureOptions = TextureOptions{
    magnification: egui::TextureFilter::Nearest,
    minification: egui::TextureFilter::Nearest
};
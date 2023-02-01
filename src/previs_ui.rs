

use std::{sync::mpsc::Receiver, time::Instant};

use eframe::App;
use egui::{Color32, Pos2, Rect, TextureHandle, Ui, Vec2, RichText, ColorImage, TextureOptions};

use crate::{LedMatrixInfo, LedFrameInfo, LedMatrixData, mapping::LedMapping};

fn black_square_image(width: usize) -> ColorImage {
    ColorImage::new([width, width], Color32::WHITE)
}

pub fn run_gui(matrices: Vec<LedMatrixInfo>, led_frame_rx: Receiver<Vec<LedMatrixData>>, led_frame_data_rx: Receiver<LedFrameInfo>) {
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

            // previs_textures_tx
            //     .send(previs_textures)
            //     .expect("Failed to send gui texture handles across threads");

            Box::new(PrevisApp::new(screens, led_frame_data_rx, led_frame_rx))
        }),
    );
}

const NEAREST_IMG_FILTER: TextureOptions = TextureOptions {
    magnification: egui::TextureFilter::Nearest,
    minification: egui::TextureFilter::Nearest,
};


pub type Screens = Vec<(LedMatrixInfo, TextureHandle)>;

fn draw_screens(ui: &mut Ui, screens: &Screens) {
    let all_cursor = ui.cursor();
    let all_offset = Vec2::new(8.0, 0.0);
    let image_scale = 10.0;

    for (screen_info, texture_handle) in screens {
        let texture_size = texture_handle.size_vec2();
        let screen_offset = screen_info.pos_offset;

        let image_offset_pos =
            Pos2::new(screen_offset.x * image_scale, screen_offset.y * image_scale)
                + all_offset * image_scale
                + all_cursor.left_top().to_vec2();
        let image_size = texture_size * image_scale;

        let image_rect = Rect::from_min_size(image_offset_pos, image_size);

        let info_text = format!(
            "universe {}\nchannel {}",
            screen_info.mapping.address.universe, screen_info.mapping.address.channel
        );

        egui::Frame::none()
            .stroke(egui::Stroke {
                width: 1.0,
                color: Color32::WHITE,
            })
            .show(&mut ui.child_ui(image_rect, *ui.layout()), |ui| {
                ui.image(texture_handle, ui.available_size());
            });

        let mut inner_ui = ui.child_ui(image_rect, ui.layout().with_main_align(egui::Align::Min));
        egui::containers::Frame::none().show(&mut inner_ui, |ui| {
            // ui.cursor().set
            // ui.put(image_rect, egui::Image::new(texture_handle, image_size));

            let _text_rect = Rect::from_center_size(image_rect.center_top(), ui.available_size());
            let label = egui::Label::new(egui::RichText::new(info_text).color(Color32::WHITE));

            // ui.with_layout(layout, add_contents)

            ui.add(label);
            // ui.put(text_rect, label);
        });
    }
}

pub struct PrevisApp {
    screens: Screens,
    frame_info_receiver: Receiver<LedFrameInfo>,
    frame_data_receiver: Receiver<Vec<LedMatrixData>>
}

impl PrevisApp {
    pub fn new(screens: Screens, frame_info_receiver: Receiver<LedFrameInfo>, frame_data_receiver: Receiver<Vec<LedMatrixData>>) -> Self {
        Self { screens, frame_info_receiver, frame_data_receiver }
    }
}

impl App for PrevisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let frame_info = self.frame_info_receiver.recv().unwrap();

        let frame_length = Instant::now() + frame_info.target_period;

        let new_frame = self.frame_data_receiver.recv().unwrap();
        
        for (data, screen) in new_frame.iter().zip(&mut self.screens) {
            let mut image = black_square_image(data.info.mapping.width);

            for (i, pixel) in data.data.iter().enumerate() {
                let pos_i = data.info.mapping.get_pos(i);
                let pixel_index: usize = pos_i.x as usize + pos_i.y as usize * image.width();

                image.pixels[pixel_index] = Color32::from_rgb(pixel[0], pixel[1], pixel[2]);

            }
            screen.1.set(image, NEAREST_IMG_FILTER)
        }
        
        let _response = egui::CentralPanel::default().show(ctx, |ui| {

            let frame_data_text = format!("target period: {:?}\nlast period: {:?}\nrendering period: {:?}", frame_info.target_period, frame_info.last_period, frame_info.rendering_period);
            ui.label(RichText::new(frame_data_text).monospace());
            draw_screens(ui, &self.screens);
        });

        // egui::Frame::none()

        ctx.request_repaint();
    }
}

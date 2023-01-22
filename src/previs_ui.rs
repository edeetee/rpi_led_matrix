use std::{thread::sleep, time::Duration};

use eframe::App;
use egui::{Color32, Pos2, Rect, TextureHandle, Ui, Vec2};

use crate::LedMatrixInfo;

pub type Screens = Vec<(LedMatrixInfo, TextureHandle)>;

fn draw_screens(ui: &mut Ui, screens: &Screens) {
    let all_cursor = ui.cursor();
    let all_offset = Vec2::new(16.0, 0.0);
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

            let text_rect = Rect::from_center_size(image_rect.center_top(), ui.available_size());
            let label = egui::Label::new(egui::RichText::new(info_text).color(Color32::WHITE));

            // ui.with_layout(layout, add_contents)

            ui.add(label);
            // ui.put(text_rect, label);
        });
    }
}

pub struct PrevisApp {
    screens: Screens,
}

impl PrevisApp {
    pub fn new(screens: Screens) -> Self {
        Self { screens }
    }
}

impl App for PrevisApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Ui::new
        let response = egui::CentralPanel::default().show(ctx, |ui| {
            draw_screens(ui, &self.screens);
        });

        ctx.request_repaint();

        // frame.size
        // ctx
        dbg!(ctx.used_size());
        // frame.set_window_size(response.response.rect.size());
    }
}



use std::{sync::mpsc::Receiver};

// use eframe::App;
use egui::{Color32, Pos2, Rect, TextureHandle, Ui, Vec2, RichText, ColorImage, TextureOptions, Context, Frame};

use egui_multiwin::{tracked_window::{TrackedWindow, RedrawResponse, TrackedWindowOptions}, multi_window::{MultiWindow, NewWindowRequest}, glutin::{event_loop, window::WindowBuilder, dpi::{PhysicalSize, LogicalSize, LogicalPosition}, platform::macos::WindowBuilderExtMacOS}};

use crate::{LedMappingInfo, LedFrameInfo, LedData, mapping::{LedMappingTrait, LedMappingEnum}, matrix_mapping::MatrixMapping};

struct InfoWindow{
    info_receiver: Receiver<LedFrameInfo>
}

impl TrackedWindow for InfoWindow {
    type Data = ();

    fn is_root(&self) -> bool {
        true
    }

    fn redraw(&mut self, data: &mut Self::Data, egui: &mut egui_multiwin::egui_glow::EguiGlow) -> egui_multiwin::tracked_window::RedrawResponse<Self::Data> {
        let frame_info = self.info_receiver.recv().unwrap();
        let frame_data_text = format!(
            "target period: {:.2}ms\nlast period: {:.2}ms\nrendering period: {:.2}ms\nlast pd message: {:.2}ms", 
            frame_info.target_period.as_secs_f32()*1000.0, 
            frame_info.last_period.as_secs_f32()*1000.0, 
            frame_info.rendering_period.as_secs_f32()*1000.0,
            frame_info.elapsed_since_pd_message.as_secs_f32()*1000.0
    );

        egui.egui_winit.set_pixels_per_point(2.0);

        let _response = egui::CentralPanel::default().show(&egui.egui_ctx, |ui| {

            ui.label(RichText::new(frame_data_text).monospace());
        });

        RedrawResponse{
            quit: false,
            new_windows: vec![]
        }
    }
}

fn new_window_request<T: TrackedWindow<Data=()> + 'static>(window: T, builder: WindowBuilder) -> NewWindowRequest<()> {
    NewWindowRequest{
        window_state: Box::new(window),

        builder: builder,
        options: TrackedWindowOptions{
            vsync: true,
            shader: None
        },
    }
}

pub fn run_gui(matrices: Vec<LedMappingInfo>, led_frame_data_rx: Receiver<Vec<LedData>>, led_frame_info_rx: Receiver<LedFrameInfo>) {

    let mut windows = MultiWindow::new();

    let event_loop = event_loop::EventLoop::default();

    let decorated_builder = WindowBuilder::new()
        // .with_decorations(false)
        .with_title("LED GUI")
        .with_always_on_top(false)
        .with_movable_by_window_background(true)
        .with_transparent(true);

    let undecorated_builder = decorated_builder.clone()
        .with_decorations(false);

    windows.add(
    new_window_request(
            InfoWindow{
                info_receiver: led_frame_info_rx
            }, 
            decorated_builder.clone()
                .with_position(LogicalPosition::new(0.0,0.0))
                .with_inner_size(LogicalSize::new(250.0, 100.0))
    ), &event_loop).unwrap();

    let fixture_group = LedFixtureGroup::new(matrices);
    let fixture_group_rect = fixture_group.screen_rect();
    println!("Fixture window of size {fixture_group_rect:#?}");
    
    windows.add(
        new_window_request(
            ScreensWindow{
                frame_data_receiver: led_frame_data_rx,
                fixtures: fixture_group
            }, 
            undecorated_builder.clone()
                .with_position(LogicalPosition::new(300.0, 0.0))
                .with_inner_size(LogicalSize::new(fixture_group_rect.width(), fixture_group_rect.height())),
        ), &event_loop).unwrap();

    windows.run(event_loop, ());
}

const NEAREST_IMG_FILTER: TextureOptions = TextureOptions {
    magnification: egui::TextureFilter::Nearest,
    minification: egui::TextureFilter::Nearest,
};

struct LedFixtureGroup {
    matrices: Vec<LedMappingInfo>,
    textures: Option<Vec<TextureHandle>>
}

const SCALE: f32 = 10.0;

fn gen_image_for_mapping(mapping: &LedMappingEnum) -> ColorImage {
    let size = mapping.get_size();
    ColorImage::new([size.x as usize, size.y as usize], Color32::WHITE)
}

impl LedFixtureGroup {
    fn new(matrices: Vec<LedMappingInfo>) -> Self {
        Self {
            matrices,
            textures: None
        }
    }

    fn screen_rect(&self) -> Rect {
        let rects = self.matrices.iter()
            .map(|info| {
                let min = Vec2::new(info.pos_offset.x, info.pos_offset.y);
                let u_size = info.mapping.get_size();
                let size = Vec2::new(u_size.x as f32, u_size.y as f32);
                Rect::from_min_size((min * SCALE).to_pos2(), size*SCALE)
            });

        // let max = positions.fold(Vec2::INFINITY, Vec2::min) + ;
        // let min = positions.fold(Vec2::ZERO, Vec2::max);

        rects.reduce(Rect::union).unwrap()
    }

    fn iter(&self) -> impl Iterator<Item=(&LedMappingInfo, &TextureHandle)> {
        self.textures.iter().flat_map(|textures| {
            textures.iter()
                .zip(self.matrices.iter())
                .map(|(handle, info)| (info, handle))
        })
    }

    fn get_textures(&mut self, ctx: &mut Context) -> &mut [TextureHandle] {
        self.textures.get_or_insert_with(|| {
            self.matrices
                .iter()
                .map(|info| {
                    ctx.load_texture(format!("img{info:?}"), gen_image_for_mapping(&info.mapping), NEAREST_IMG_FILTER)
                })
                .collect()
        })
    }
}

fn draw_screens(ui: &mut Ui, group: &LedFixtureGroup) {
    let all_cursor = ui.cursor();
    // let all_offset = group.screen_rect()
    let group_offset = group.screen_rect().left_top().to_vec2();
    // let min_offset = group.re
    let image_scale = 10.0;

    for (screen_info, texture_handle) in group.iter() {
        let texture_size = texture_handle.size_vec2();
        let screen_offset = screen_info.pos_offset;

        let image_offset_pos =
            Pos2::new(screen_offset.x * SCALE, screen_offset.y * SCALE)
                - group_offset
                + all_cursor.left_top().to_vec2();
                
        let image_size = texture_size * image_scale;

        let image_rect = Rect::from_min_size(image_offset_pos, image_size);

        let info_text = format!(
            "u: {}\nc: {}",
            screen_info.dmx_address.universe, screen_info.dmx_address.channel
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

pub struct ScreensWindow {
    fixtures: LedFixtureGroup,
    frame_data_receiver: Receiver<Vec<LedData>>
}

impl TrackedWindow for ScreensWindow {
    type Data = ();

    fn redraw(&mut self, _c: &mut Self::Data, egui: &mut egui_multiwin::egui_glow::EguiGlow) -> RedrawResponse<Self::Data> {
        // frame.drag_window();
        egui.egui_winit.set_pixels_per_point(2.0);

        let new_frame = self.frame_data_receiver.recv().unwrap();

        let textures =  self.fixtures.get_textures(&mut egui.egui_ctx);

        for (data, screen) in new_frame.iter().zip(textures.iter_mut()) {
            let mut image = gen_image_for_mapping(&data.info.mapping);

            for (i, pixel) in data.data.iter().enumerate() {
                let pos_i = data.info.mapping.get_pos(i);
                let pixel_index: usize = pos_i.x as usize + pos_i.y as usize * image.width();

                image.pixels[pixel_index] = Color32::from_rgb(pixel[0], pixel[1], pixel[2]);

            }
            screen.set(image, NEAREST_IMG_FILTER)
        }
        
        let _response = egui::CentralPanel::default()
            .frame(Frame::none().fill(Color32::BLACK))
            .show(&egui.egui_ctx, |ui| {
                draw_screens(ui, &self.fixtures);
            });

        // egui::Frame::none()

        egui.egui_ctx.request_repaint();

        RedrawResponse { quit: false, new_windows: vec![] }
    }
}

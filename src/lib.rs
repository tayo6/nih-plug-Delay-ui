use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;

// ==========================================================================
// VST3 PLUGIN SETUP
// ==========================================================================

pub struct DelayVst {
    params: Arc<DelayVstParams>,
}

#[derive(Params)]
struct DelayVstParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,
}

impl Default for DelayVst {
    fn default() -> Self {
        Self {
            params: Arc::new(DelayVstParams {
                editor_state: EguiState::from_size(570, 480),
            }),
        }
    }
}

impl Plugin for DelayVst {
    const NAME: &'static str = "Delay VST Replica";
    const VENDOR: &'static str = "Tayo6";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = "1.0.0";
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let mut app_state = DelayVstApp::default();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            // Updated for nih-plug's new 3-argument closure
            move |_, ctx, _setter| {
                app_state.update(ctx);
            },
        )
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Audio passes through cleanly. (DSP logic goes here later!)
        ProcessStatus::Normal
    }
}

// Fixed: Removed the accidental 'impl' keyword here
nih_plug::nih_export_vst3!(DelayVst);

// ==========================================================================
// APP REPRESENTATION & IMPLEMENTATION
// ==========================================================================

struct DelayVstApp {
    tempo_index: usize,
    tempo_drag_accumulator: f32,
    regen_value: f32,
    mix_value: f32,
    output_value: f32,

    studio_mode: bool,
    auto_gain: bool,
    brightness_active: bool,
    color_active: bool,
    sparkle_active: bool,

    active_level_in: f32,
    active_level_out: f32,
}

impl Default for DelayVstApp {
    fn default() -> Self {
        Self {
            tempo_index: 2, 
            tempo_drag_accumulator: 0.0,
            regen_value: 0.722,
            mix_value: 0.537,
            output_value: 0.444,
            studio_mode: true,
            auto_gain: true,
            brightness_active: false,
            color_active: false,
            sparkle_active: false,
            active_level_in: 0.0,
            active_level_out: 0.0,
        }
    }
}

impl DelayVstApp {
    fn update(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();

        let screen_rect = ctx.screen_rect();
        let current_zoom = ctx.zoom_factor();
        let unscaled_width = screen_rect.width() * current_zoom;
        let target_width = 570.0;
        let scale = (unscaled_width / target_width).min(1.0);
        if (scale - current_zoom).abs() > 0.01 {
            ctx.set_zoom_factor(scale);
        }

        let time = ctx.input(|i| i.time) as f32;
        let target_in = (0.50f32 + (time * 1.8f32).sin().abs() * 0.22f32 + (time * 3.3f32).cos().abs() * 0.12f32).clamp(0.0f32, 1.0f32);
        let target_out = (0.45f32 + (time * 1.3f32).cos().abs() * 0.32f32 + (time * 4.2f32).sin().abs() * 0.15f32).clamp(0.0f32, 1.0f32);

        if target_in > self.active_level_in {
            self.active_level_in += (target_in - self.active_level_in) * 0.35f32;
        } else {
            self.active_level_in += (target_in - self.active_level_in) * 0.12f32;
        }

        if target_out > self.active_level_out {
            self.active_level_out += (target_out - self.active_level_out) * 0.35f32;
        } else {
            self.active_level_out += (target_out - self.active_level_out) * 0.12f32;
        }

        egui::CentralPanel::default()
            // Updated to Frame::NONE for egui 0.31
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(18, 21, 27)))
            .show(ctx, |ui| {
                let full_rect = ui.max_rect();
                let painter = ui.painter();

                painter.circle_filled(
                    full_rect.center(),
                    260.0,
                    egui::Color32::from_rgba_unmultiplied(27, 32, 42, 120),
                );

                let vst_rect = egui::Rect::from_center_size(full_rect.center(), egui::vec2(530.0, 440.0));

                for i in 1..=6 {
                    let shadow_rect = vst_rect.expand(i as f32 * 1.8);
                    painter.rect_stroke(
                        shadow_rect,
                        // Updated to CornerRadius and u8
                        egui::CornerRadius::same(8 + i as u8),
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, (30 / i) as u8)),
                        // Added StrokeKind for egui 0.31
                        egui::StrokeKind::Middle,
                    );
                }

                painter.rect_filled(vst_rect, egui::CornerRadius::same(8), egui::Color32::from_rgb(245, 247, 250));
                painter.rect_stroke(
                    vst_rect, 
                    egui::CornerRadius::same(8), 
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(187, 196, 204)),
                    egui::StrokeKind::Middle,
                );

                ui.allocate_ui_at_rect(vst_rect, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    let top_rect = egui::Rect::from_min_size(vst_rect.min, egui::vec2(530.0, 160.0));
                    let mid_rect = egui::Rect::from_min_size(vst_rect.min + egui::vec2(0.0, 160.0), egui::vec2(530.0, 38.0));
                    let bottom_rect = egui::Rect::from_min_size(vst_rect.min + egui::vec2(0.0, 198.0), egui::vec2(530.0, 242.0));

                    draw_top_panel(ui, top_rect, self, time);
                    draw_mid_bar(ui, mid_rect, self);
                    draw_bottom_panel(ui, bottom_rect, self, ctx);
                });
            });
    }
}

fn draw_top_panel(ui: &mut egui::Ui, rect: egui::Rect, _app: &DelayVstApp, time: f32) {
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        egui::CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 },
        egui::Color32::from_rgb(9, 14, 18),
    );

    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)),
    );

    let logo_pos = rect.min + egui::vec2(20.0, 16.0);
    painter.text(
        logo_pos,
        egui::Align2::LEFT_TOP,
        "DELAY ▼",
        egui::FontId::proportional(13.0),
        egui::Color32::from_rgb(44, 229, 196),
    );

    let shift_x = (rect.width() - 380.0) / 2.0;

    let draw_glowing_line = |p: &egui::Painter, p1: egui::Pos2, p2: egui::Pos2, stroke: egui::Stroke| {
        // Fixed: Added the missing [ ] brackets around p1 and p2
        p.line_segment([p1, p2], egui::Stroke::new(stroke.width + 3.0, egui::Color32::from_rgba_unmultiplied(44, 229, 196, 25)));
        p.line_segment([p1, p2], egui::Stroke::new(stroke.width + 1.5, egui::Color32::from_rgba_unmultiplied(44, 229, 196, 60)));
        p.line_segment([p1, p2], stroke);
    };

    let left_center = egui::vec2(150.0, 87.0);
    let left_offset_y = (time * 1.5f32).sin() * 4.0f32;
    let left_breath_scale = 1.0f32 + ((time * 1.5f32).sin() * 0.015f32);
    let map_pt_large = |x: f32, y: f32| -> egui::Pos2 {
        let rel_v = egui::vec2(x, y) - left_center;
        let scaled_v = rel_v * left_breath_scale;
        rect.min + egui::vec2(left_center.x + scaled_v.x + shift_x, left_center.y + scaled_v.y + left_offset_y)
    };

    let stroke_large = egui::Stroke::new(1.8, egui::Color32::from_rgb(44, 229, 196));

    let p_t1 = map_pt_large(150.0, 38.0);
    let p_t2 = map_pt_large(185.0, 58.0);
    let p_t3 = map_pt_large(150.0, 78.0);
    let p_t4 = map_pt_large(115.0, 58.0);

    draw_glowing_line(painter, p_t1, p_t2, stroke_large);
    draw_glowing_line(painter, p_t2, p_t3, stroke_large);
    draw_glowing_line(painter, p_t3, p_t4, stroke_large);
    draw_glowing_line(painter, p_t4, p_t1, stroke_large);

    let p_b1 = map_pt_large(150.0, 96.0);
    let p_b2 = map_pt_large(185.0, 116.0);
    let p_b3 = map_pt_large(150.0, 136.0);
    let p_b4 = map_pt_large(115.0, 116.0);

    draw_glowing_line(painter, p_b1, p_b2, stroke_large);
    draw_glowing_line(painter, p_b2, p_b3, stroke_large);
    draw_glowing_line(painter, p_b3, p_b4, stroke_large);
    draw_glowing_line(painter, p_b4, p_b1, stroke_large);

    draw_glowing_line(painter, p_t1, p_b1, stroke_large);
    draw_glowing_line(painter, p_t2, p_b2, stroke_large);
    draw_glowing_line(painter, p_t3, p_b3, stroke_large);
    draw_glowing_line(painter, p_t4, p_b4, stroke_large);

    let right_center = egui::vec2(255.0, 82.0);
    let right_offset_y = ((time - 3.0f32) * 1.5f32).sin() * 4.0f32;
    let right_breath_scale = 1.0f32 + (((time - 3.0f32) * 1.5f32).sin() * 0.015f32);
    let map_pt_small = |x: f32, y: f32| -> egui::Pos2 {
        let rel_v = egui::vec2(x, y) - right_center;
        let scaled_v = rel_v * right_breath_scale;
        rect.min + egui::vec2(right_center.x + scaled_v.x + shift_x, right_center.y + scaled_v.y + right_offset_y)
    };

    let stroke_small = egui::Stroke::new(1.5, egui::Color32::from_rgb(44, 229, 196));

    let p_st1 = map_pt_small(255.0, 54.0);
    let p_st2 = map_pt_small(276.0, 66.0);
    let p_st3 = map_pt_small(255.0, 78.0);
    let p_st4 = map_pt_small(234.0, 66.0);

    draw_glowing_line(painter, p_st1, p_st2, stroke_small);
    draw_glowing_line(painter, p_st2, p_st3, stroke_small);
    draw_glowing_line(painter, p_st3, p_st4, stroke_small);
    draw_glowing_line(painter, p_st4, p_st1, stroke_small);

    let p_sb1 = map_pt_small(255.0, 86.0);
    let p_sb2 = map_pt_small(276.0, 98.0);
    let p_sb3 = map_pt_small(255.0, 110.0);
    let p_sb4 = map_pt_small(234.0, 98.0);

    draw_glowing_line(painter, p_sb1, p_sb2, stroke_small);
    draw_glowing_line(painter, p_sb2, p_sb3, stroke_small);
    draw_glowing_line(painter, p_sb3, p_sb4, stroke_small);
    draw_glowing_line(painter, p_sb4, p_sb1, stroke_small);

    draw_glowing_line(painter, p_st1, p_sb1, stroke_small);
    draw_glowing_line(painter, p_st2, p_sb2, stroke_small);
    draw_glowing_line(painter, p_st3, p_sb3, stroke_small);
    draw_glowing_line(painter, p_st4, p_sb4, stroke_small);
}

fn draw_mid_bar(ui: &mut egui::Ui, rect: egui::Rect, app: &mut DelayVstApp) {
    let toggle_area = egui::Rect::from_min_max(
        rect.left_top() + egui::vec2(24.0, 5.0),
        rect.left_top() + egui::vec2(180.0, 33.0),
    );
    let toggle_response = ui.allocate_rect(toggle_area, egui::Sense::click());
    if toggle_response.clicked() {
        app.studio_mode = !app.studio_mode;
    }

    let auto_gain_area = egui::Rect::from_min_max(
        rect.right_top() + egui::vec2(-110.0, 5.0),
        rect.right_top() + egui::vec2(-24.0, 33.0),
    );
    let ag_response = ui.allocate_rect(auto_gain_area, egui::Sense::click());
    if ag_response.clicked() {
        app.auto_gain = !app.auto_gain;
    }

    let painter = ui.painter();

    painter.rect_filled(rect, egui::CornerRadius::same(0), egui::Color32::from_rgb(241, 243, 246));
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(225, 228, 232)),
    );

    let studio_color = if app.studio_mode { egui::Color32::from_rgb(46, 53, 64) } else { egui::Color32::from_rgb(138, 148, 166) };
    let creative_color = if !app.studio_mode { egui::Color32::from_rgb(46, 53, 64) } else { egui::Color32::from_rgb(138, 148, 166) };

    painter.text(
        rect.left_top() + egui::vec2(24.0, 19.0),
        egui::Align2::LEFT_CENTER,
        "STUDIO",
        egui::FontId::proportional(9.0),
        studio_color,
    );

    let switch_rect = egui::Rect::from_min_size(rect.left_top() + egui::vec2(74.0, 10.5), egui::vec2(32.0, 17.0));
    let switch_bg = if app.studio_mode { egui::Color32::from_rgb(142, 153, 252) } else { egui::Color32::from_rgb(203, 213, 224) };
    painter.rect_filled(switch_rect, egui::CornerRadius::same(9), switch_bg);

    painter.line_segment(
        [switch_rect.left_top() + egui::vec2(2.0, 1.0), switch_rect.right_top() + egui::vec2(-2.0, 1.0)],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 45)),
    );

    let handle_x = if app.studio_mode { switch_rect.left() + 2.0 } else { switch_rect.left() + 17.0 };
    let handle_rect = egui::Rect::from_min_size(egui::pos2(handle_x, switch_rect.top() + 2.0), egui::vec2(13.0, 13.0));
    painter.rect_filled(handle_rect, egui::CornerRadius::same(6), egui::Color32::WHITE);

    painter.text(
        rect.left_top() + egui::vec2(116.0, 19.0),
        egui::Align2::LEFT_CENTER,
        "CREATIVE",
        egui::FontId::proportional(9.0),
        creative_color,
    );

    painter.text(
        rect.right_top() + egui::vec2(-48.0, 14.0),
        egui::Align2::RIGHT_CENTER,
        "AUTO",
        egui::FontId::proportional(8.0),
        egui::Color32::from_rgb(74, 85, 104),
    );
    painter.text(
        rect.right_top() + egui::vec2(-48.0, 24.0),
        egui::Align2::RIGHT_CENTER,
        "GAIN",
        egui::FontId::proportional(8.0),
        egui::Color32::from_rgb(74, 85, 104),
    );

    let led_rect = egui::Rect::from_min_size(rect.right_top() + egui::vec2(-42.0, 13.5), egui::vec2(18.0, 11.0));
    let led_color = if app.auto_gain {
        egui::Color32::from_rgb(89, 243, 140)
    } else {
        egui::Color32::from_rgb(160, 174, 192)
    };
    painter.rect_filled(led_rect, egui::CornerRadius::same(2), led_color);
}

fn draw_bottom_panel(ui: &mut egui::Ui, rect: egui::Rect, app: &mut DelayVstApp, ctx: &egui::Context) {
    let divider_x = rect.left() + 413.0;
    let cx = divider_x + 58.5;

    let tempo_center = egui::pos2(rect.left() + 57.0, rect.top() + 65.0);
    let regen_center = egui::pos2(rect.left() + 205.5, rect.top() + 65.0);
    let mix_center = egui::pos2(rect.left() + 354.0, rect.top() + 65.0);

    let btn_width = 85.0;
    let btn_height = 64.0;
    let btn_y = rect.top() + 175.0;

    let bright_rect = egui::Rect::from_center_size(egui::pos2(rect.left() + 66.5, btn_y), egui::vec2(btn_width, btn_height));
    let color_rect = egui::Rect::from_center_size(egui::pos2(rect.left() + 206.5, btn_y), egui::vec2(btn_width, btn_height));
    let sparkle_rect = egui::Rect::from_center_size(egui::pos2(rect.left() + 346.5, btn_y), egui::vec2(btn_width, btn_height));

    let tempo_rect = egui::Rect::from_center_size(tempo_center, egui::vec2(66.0, 66.0));
    let tempo_response = ui.allocate_rect(tempo_rect, egui::Sense::click_and_drag());
    if tempo_response.dragged() {
        let dy = tempo_response.drag_delta().y;
        if dy.abs() > 1.5f32 {
            app.tempo_drag_accumulator += dy;
            if app.tempo_drag_accumulator > 15.0f32 {
                if app.tempo_index > 0 { app.tempo_index -= 1; }
                app.tempo_drag_accumulator = 0.0f32;
            } else if app.tempo_drag_accumulator < -15.0f32 {
                if app.tempo_index < tempo_values_len() { app.tempo_index += 1; }
                app.tempo_drag_accumulator = 0.0f32;
            }
        }
    } else {
        app.tempo_drag_accumulator = 0.0f32;
    }

    let regen_rect = egui::Rect::from_center_size(regen_center, egui::vec2(52.0, 52.0));
    let regen_response = ui.allocate_rect(regen_rect, egui::Sense::click_and_drag());
    if regen_response.dragged() {
        let dy = regen_response.drag_delta().y;
        app.regen_value = (app.regen_value - dy * 0.005f32).clamp(0.0f32, 1.0f32);
    }

    let mix_rect = egui::Rect::from_center_size(mix_center, egui::vec2(68.0, 68.0));
    let mix_response = ui.allocate_rect(mix_rect, egui::Sense::click_and_drag());
    if mix_response.dragged() {
        let dy = mix_response.drag_delta().y;
        app.mix_value = (app.mix_value - dy * 0.005f32).clamp(0.0f32, 1.0f32);
    }

    let bright_response = ui.allocate_rect(bright_rect, egui::Sense::click());
    if bright_response.clicked() {
        app.brightness_active = !app.brightness_active;
    }

    let color_response = ui.allocate_rect(color_rect, egui::Sense::click());
    if color_response.clicked() {
        app.color_active = !app.color_active;
    }

    let sparkle_response = ui.allocate_rect(sparkle_rect, egui::Sense::click());
    if sparkle_response.clicked() {
        app.sparkle_active = !app.sparkle_active;
    }

    let out_knob_center = egui::pos2(cx, rect.top() + 175.0);
    let out_knob_rect = egui::Rect::from_center_size(out_knob_center, egui::vec2(52.0, 52.0));
    let out_knob_response = ui.allocate_rect(out_knob_rect, egui::Sense::click_and_drag());
    if out_knob_response.dragged() {
        let dy = out_knob_response.drag_delta().y;
        app.output_value = (app.output_value - dy * 0.005f32).clamp(0.0f32, 1.0f32);
    }

    if tempo_response.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }
    if regen_response.hovered() {
        ctx.set_cursor_icon(if regen_response.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
    }
    if mix_response.hovered() {
        ctx.set_cursor_icon(if mix_response.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
    }
    if bright_response.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if color_response.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if sparkle_response.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if out_knob_response.hovered() {
        ctx.set_cursor_icon(if out_knob_response.dragged() { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
    }

    let painter = ui.painter();

    painter.rect_filled(rect, egui::CornerRadius { nw: 0, ne: 0, sw: 8, se: 8 }, egui::Color32::from_rgb(247, 249, 250));
    painter.line_segment(
        [egui::pos2(divider_x, rect.top()), egui::pos2(divider_x, rect.bottom())],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(225, 228, 232)),
    );

    painter.line_segment(
        [rect.left_top() + egui::vec2(0.0, 1.0), rect.right_top() + egui::vec2(0.0, 1.0)],
        egui::Stroke::new(1.5, egui::Color32::WHITE),
    );

    let tempo_values = ["1/32", "1/16", "1/8", "1/4", "1/2", "1/1"];
    painter.circle_stroke(tempo_center, 28.0, egui::Stroke::new(3.5, egui::Color32::from_rgb(226, 232, 240)));

    let progress = (app.tempo_index + 1) as f32 / tempo_values.len() as f32;
    let stroke_color = egui::Color32::from_rgb(44, 229, 196);

    let start_angle = -std::f32::consts::FRAC_PI_2;
    let sweep_angle = progress * 2.0f32 * std::f32::consts::PI;
    let num_segments = 32;
    let mut last_point = tempo_center + egui::vec2(start_angle.cos(), start_angle.sin()) * 28.0;
    for i in 1..=num_segments {
        let t = i as f32 / num_segments as f32;
        let angle = start_angle + sweep_angle * t;
        let next_point = tempo_center + egui::vec2(angle.cos(), angle.sin()) * 28.0;
        painter.line_segment([last_point, next_point], egui::Stroke::new(3.5, stroke_color));
        last_point = next_point;
    }

    painter.text(
        tempo_center,
        egui::Align2::CENTER_CENTER,
        tempo_values[app.tempo_index],
        egui::FontId::proportional(15.0),
        egui::Color32::from_rgb(45, 55, 72),
    );

    painter.text(
        tempo_center + egui::vec2(0.0, 48.0),
        egui::Align2::CENTER_CENTER,
        "TEMPO",
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(113, 128, 150),
    );

    draw_knob(painter, regen_center, 22.0, app.regen_value);
    painter.text(
        regen_center + egui::vec2(0.0, 48.0),
        egui::Align2::CENTER_CENTER,
        "REGEN",
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(113, 128, 150),
    );

    draw_knob(painter, mix_center, 29.0, app.mix_value);
    painter.text(
        mix_center + egui::vec2(0.0, 48.0),
        egui::Align2::CENTER_CENTER,
        "MIX",
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(113, 128, 150),
    );

    draw_filter_button(painter, bright_rect, "BRIGHTNESS", app.brightness_active, |p, center| {
        let radius = 12.0;
        let stroke_stripe = egui::Stroke::new(1.3, if app.brightness_active { egui::Color32::from_rgb(44, 229, 196) } else { egui::Color32::from_rgb(113, 128, 150) });
        p.circle_stroke(center, radius, stroke_stripe);

        let step = 2.5;
        let mut y_offset = -radius + 1.5;
        while y_offset < radius {
            let half_chord = (radius * radius - y_offset * y_offset).sqrt();
            let p1 = center + egui::vec2(-half_chord, y_offset);
            let p2 = center + egui::vec2(0.0, y_offset);
            p.line_segment([p1, p2], stroke_stripe);
            y_offset += step;
        }
        p.line_segment([center + egui::vec2(0.0, -radius), center + egui::vec2(0.0, radius)], stroke_stripe);
    });

    draw_filter_button(painter, color_rect, "COLOR", app.color_active, |p, center| {
        let stroke = egui::Stroke::new(1.4, if app.color_active { egui::Color32::from_rgb(44, 229, 196) } else { egui::Color32::from_rgb(113, 128, 150) });
        let r_circle = 5.0;
        p.circle_stroke(center, r_circle, stroke);
        for i in 0..6 {
            let angle = (i as f32 * 60.0).to_radians();
            let offset = egui::vec2(angle.cos(), angle.sin()) * 5.5;
            p.circle_stroke(center + offset, r_circle, stroke);
        }
    });

    draw_filter_button(painter, sparkle_rect, "SPARKLE", app.sparkle_active, |p, center| {
        let color = if app.sparkle_active { egui::Color32::from_rgb(44, 229, 196) } else { egui::Color32::from_rgb(113, 128, 150) };

        let draw_quadratic_bezier_sparkle = |p_paint: &egui::Painter, c: egui::Pos2, radius: f32| {
            let mut points = Vec::new();
            let tips = [
                egui::vec2(0.0, -radius),
                egui::vec2(radius, 0.0),
                egui::vec2(0.0, radius),
                egui::vec2(-radius, 0.0),
            ];
            
            let num_samples = 6;
            for i in 0..4 {
                let p0 = tips[i];
                let p2 = tips[(i + 1) % 4];
                for step in 0..num_samples {
                    let t = step as f32 / num_samples as f32;
                    let b = p0 * (1.0 - t).powi(2) + p2 * t.powi(2);
                    points.push(c + b);
                }
            }
            p_paint.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
        };

        let stars = [
            (egui::vec2(0.0, -10.0), 3.85),
            (egui::vec2(-6.0, -2.0), 2.97),
            (egui::vec2(6.0, -1.0), 3.32),
            (egui::vec2(0.0, 7.0), 4.02),
            (egui::vec2(-8.0, -8.0), 2.1),
            (egui::vec2(8.0, -9.0), 2.1),
            (egui::vec2(-8.0, 5.0), 2.1),
            (egui::vec2(8.0, 6.0), 2.27),
        ];

        for (offset, rad) in stars {
            draw_quadratic_bezier_sparkle(p, center + offset, rad);
        }
    });

    let draw_vu_meter = |p: &egui::Painter, meter_cx: f32, level: f32, label: &str| {
        let meter_rect = egui::Rect::from_center_size(
            egui::pos2(meter_cx, rect.top() + 55.0),
            egui::vec2(11.0, 86.5),
        );
        p.rect_filled(meter_rect, egui::CornerRadius::same(2), egui::Color32::from_rgb(226, 232, 240));

        p.rect_stroke(
            meter_rect, 
            egui::CornerRadius::same(2), 
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30)),
            egui::StrokeKind::Middle,
        );

        let lit_limit = (level * 24.0f32).round() as usize;
        let led_w = 7.0;
        let led_h = 2.0;
        let gap = 1.5;

        for i in 0..24 {
            let y = meter_rect.bottom() - 2.0 - i as f32 * (led_h + gap);
            let led_rect = egui::Rect::from_center_size(egui::pos2(meter_cx, y), egui::vec2(led_w, led_h));
            let is_lit = i < lit_limit;

            let led_color = if is_lit {
                if i < 16 {
                    egui::Color32::from_rgb(82, 236, 135)
                } else if i < 19 {
                    egui::Color32::from_rgb(76, 223, 242)
                } else if i < 22 {
                    egui::Color32::from_rgb(252, 163, 61)
                } else {
                    egui::Color32::from_rgb(255, 94, 94)
                }
            } else {
                egui::Color32::from_rgb(203, 213, 220)
            };
            p.rect_filled(led_rect, egui::CornerRadius::same(0), led_color);
        }

        p.text(
            egui::pos2(meter_cx, rect.top() + 112.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(8.0),
            egui::Color32::from_rgb(113, 128, 150),
        );
    };

    draw_vu_meter(painter, cx - 12.5, app.active_level_in, "IN");
    draw_vu_meter(painter, cx + 12.5, app.active_level_out, "OUT");

    draw_knob(painter, out_knob_center, 22.0, app.output_value);
    painter.text(
        out_knob_center + egui::vec2(0.0, 48.0),
        egui::Align2::CENTER_CENTER,
        "OUTPUT",
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(113, 128, 150),
    );
}

fn tempo_values_len() -> usize {
    5
}

fn draw_knob(painter: &egui::Painter, center: egui::Pos2, radius: f32, val: f32) {
    let num_dots = 16;
    for i in 0..num_dots {
        let angle = (i as f32 * 360.0 / num_dots as f32).to_radians();
        let dot_pos = center + egui::vec2(angle.cos(), angle.sin()) * (radius + 4.0);
        painter.circle_filled(dot_pos, 1.0, egui::Color32::from_rgb(160, 174, 192).linear_multiply(0.45));
    }

    let shadow_offset = egui::vec2(0.0, 3.0);
    painter.circle_filled(center + shadow_offset, radius + 3.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 10));
    painter.circle_filled(center + shadow_offset * 0.7, radius + 1.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 18));
    painter.circle_filled(center + shadow_offset * 0.4, radius, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 24));

    painter.circle_filled(center, radius, egui::Color32::from_rgb(184, 200, 213));

    painter.circle_filled(center - egui::vec2(0.0, 1.0), radius - 1.2, egui::Color32::from_rgb(160, 175, 190));
    painter.circle_filled(center + egui::vec2(0.0, 1.0), radius - 1.2, egui::Color32::WHITE.linear_multiply(0.85));

    painter.circle_filled(center, radius - 2.5, egui::Color32::from_rgb(203, 219, 231));
    
    let shine_offset = egui::vec2(-radius * 0.15, -radius * 0.15);
    painter.circle_filled(center + shine_offset, (radius - 2.5) * 0.85, egui::Color32::from_rgb(241, 246, 250));

    let min_angle = -135.0f32.to_radians();
    let max_angle = 135.0f32.to_radians();
    let rotation_angle = min_angle + val * (max_angle - min_angle);
    let angle_rad = -std::f32::consts::FRAC_PI_2 + rotation_angle;

    let pointer_dist = radius - 5.5;
    let pointer_pos = center + egui::vec2(angle_rad.cos(), angle_rad.sin()) * pointer_dist;
    painter.circle_filled(pointer_pos, 2.0, egui::Color32::from_rgb(74, 85, 104));
}

fn draw_filter_button<F>(painter: &egui::Painter, rect: egui::Rect, label: &str, active: bool, draw_icon: F)
where
    F: FnOnce(&egui::Painter, egui::Pos2),
{
    let bg_color = if active {
        egui::Color32::from_rgb(235, 253, 250)
    } else {
        egui::Color32::TRANSPARENT
    };
    let border_stroke = if active {
        egui::Stroke::new(1.2, egui::Color32::from_rgb(44, 229, 196))
    } else {
        egui::Stroke::NONE
    };

    painter.rect(
        rect, 
        egui::CornerRadius::same(6), 
        bg_color, 
        border_stroke,
        egui::StrokeKind::Middle,
    );

    let icon_center = egui::pos2(rect.center().x, rect.top() + 22.0);
    draw_icon(painter, icon_center);

    let label_color = if active {
        egui::Color32::from_rgb(44, 229, 196)
    } else {
        egui::Color32::from_rgb(113, 128, 150)
    };
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 10.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(8.0),
        label_color,
    );
}
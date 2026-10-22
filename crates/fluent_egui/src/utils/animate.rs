use std::time::Duration;

use egui::{Color32, Context, Id};

/// Smoothly moves a color towards `target` over `duration`, like Flutter's
/// `AnimatedContainer`. Interpolates premultiplied components.
pub fn animate_color(ctx: &Context, id: Id, target: Color32, duration: Duration) -> Color32 {
    let secs = duration.as_secs_f32();
    let [r, g, b, a] = target.to_array().map(f32::from);
    let channel = |i: u8, v: f32| ctx.animate_value_with_time(id.with(i), v, secs).round() as u8;
    Color32::from_rgba_premultiplied(channel(0, r), channel(1, g), channel(2, b), channel(3, a))
}

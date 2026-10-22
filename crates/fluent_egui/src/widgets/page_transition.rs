use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use egui::emath::TSTransform;
use egui::layers::ShapeIdx;
use egui::{AsIdSalt, IdSalt, InnerResponse, Sense, Ui, UiBuilder, vec2};

use crate::FluentTheme;

/// How a new page animates in, fluent_ui's page transition widgets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PageTransitionKind {
    /// Slides in from below (or the side) while fading in,
    /// `EntrancePageTransition`. The default for a [`super::NavigationView`].
    Entrance {
        vertical: bool,
        reverse: bool,
        /// How far away the page starts, as a fraction of its size.
        start_from: f32,
    },
    /// Grows from 88% while fading in, `DrillInPageTransition`. Suits moving
    /// deeper into an app.
    DrillIn,
    /// Slides in from the side, `HorizontalSlidePageTransition`. The default
    /// for a top navigation bar.
    HorizontalSlide { from_left: bool },
    /// No animation, `SuppressPageTransition`.
    Suppress,
}

/// Animates `contents` whenever the page key changes.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, page: usize) {
/// use fluent_egui::PageTransition;
///
/// PageTransition::drill_in().show(ui, "pages", page, |ui| {
///     ui.label(format!("Page {page}"));
/// });
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PageTransition {
    pub kind: PageTransitionKind,
    /// Defaults to the theme's fast animation duration.
    pub duration: Option<Duration>,
}

#[derive(Clone, Copy)]
struct TransitionState {
    key: u64,
    start: f64,
}

impl Default for PageTransition {
    fn default() -> Self {
        Self::entrance()
    }
}

impl PageTransition {
    pub fn new(kind: PageTransitionKind) -> Self {
        Self {
            kind,
            duration: None,
        }
    }

    pub fn entrance() -> Self {
        Self::new(PageTransitionKind::Entrance {
            vertical: true,
            reverse: false,
            start_from: 0.25,
        })
    }

    pub fn drill_in() -> Self {
        Self::new(PageTransitionKind::DrillIn)
    }

    pub fn horizontal_slide(from_left: bool) -> Self {
        Self::new(PageTransitionKind::HorizontalSlide { from_left })
    }

    pub fn suppress() -> Self {
        Self::new(PageTransitionKind::Suppress)
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Shows `contents` in the rest of `ui`, animating it in when `key`
    /// differs from last frame's. The first frame shows no animation.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        id_salt: impl AsIdSalt,
        key: impl Hash,
        contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let theme = FluentTheme::of(ui.ctx());
        let id = ui.make_persistent_id(IdSalt::new(id_salt));
        let now = ui.input(|i| i.time);
        let key = {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            hasher.finish()
        };
        let state = match ui.data(|d| d.get_temp::<TransitionState>(id)) {
            Some(state) if state.key == key => state,
            Some(_) => TransitionState { key, start: now },
            // Nothing to transition from on the first frame.
            None => TransitionState {
                key,
                start: f64::NEG_INFINITY,
            },
        };
        ui.data_mut(|d| d.insert_temp(id, state));

        let duration = self
            .duration
            .unwrap_or(theme.fast_animation_duration)
            .as_secs_f64()
            .max(1e-3);
        let linear = ((now - state.start) / duration).clamp(0.0, 1.0) as f32;
        if linear < 1.0 {
            ui.request_repaint();
        }
        let t = (theme.animation_curve)(linear);

        let rect = ui.available_rect_before_wrap();
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(*ui.layout()));
        let fades = matches!(
            self.kind,
            PageTransitionKind::Entrance { .. } | PageTransitionKind::DrillIn
        );
        if fades && t < 1.0 {
            child.multiply_opacity(t);
        }

        let layer = child.layer_id();
        let first = ui
            .ctx()
            .graphics(|g| g.get(layer).map_or(0, |l| l.next_idx().0));
        let inner = contents(&mut child);

        if t < 1.0 {
            let transform = match self.kind {
                PageTransitionKind::Entrance {
                    vertical,
                    reverse,
                    start_from,
                } => {
                    // fluent_ui slides from `value + start_from` to zero.
                    let sign = if reverse { -1.0 } else { 1.0 };
                    let fraction = (1.0 - t) * (t + start_from) * sign;
                    let offset = if vertical {
                        vec2(0.0, fraction * rect.height())
                    } else {
                        vec2(fraction * rect.width(), 0.0)
                    };
                    TSTransform::from_translation(offset)
                }
                PageTransitionKind::DrillIn => {
                    let scale = egui::lerp(0.88..=1.0, t);
                    let c = rect.center().to_vec2();
                    TSTransform::from_translation(c)
                        * TSTransform::from_scaling(scale)
                        * TSTransform::from_translation(-c)
                }
                PageTransitionKind::HorizontalSlide { from_left } => {
                    let sign = if from_left { -1.0 } else { 1.0 };
                    TSTransform::from_translation(vec2(sign * 0.65 * (1.0 - t) * rect.width(), 0.0))
                }
                PageTransitionKind::Suppress => TSTransform::IDENTITY,
            };
            if transform != TSTransform::IDENTITY {
                // Move the shapes but keep their clip rects, so the page stays
                // inside its area while it moves.
                ui.ctx().graphics_mut(|g| {
                    let list = g.entry(layer);
                    for i in first..list.next_idx().0 {
                        list.mutate_shape(ShapeIdx(i), |s| s.shape.transform(transform));
                    }
                });
            }
        }

        let response = ui.allocate_rect(child.min_rect(), Sense::hover());
        InnerResponse::new(inner, response)
    }
}

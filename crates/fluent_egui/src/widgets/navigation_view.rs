use egui::{
    Align, Area, AsIdSalt, CornerRadius, IdSalt, Layout, Order, Pos2, Rect, Response, Role,
    ScrollArea, Sense, Shadow, Stroke, StrokeKind, TextWrapMode, Ui, UiBuilder, Vec2, WidgetInfo,
    WidgetText, pos2, vec2,
};

use super::FluentTooltip as _;
use super::PageTransition;
use crate::FluentTheme;
use crate::Localizations;
use crate::utils::{
    ControlState, States, animate_color, paint_focus_border, paint_hamburger, with_alpha,
};

/// Width of the pane in compact mode, `kCompactNavigationPaneWidth`.
const COMPACT_WIDTH: f32 = 50.0;
/// Width of the open pane, `kOpenNavigationPaneWidth`.
const OPEN_WIDTH: f32 = 320.0;
/// Below this width [`PaneDisplayMode::Auto`] is minimal, `kCompactModeThresholdWidth`.
const COMPACT_THRESHOLD: f32 = 640.0;
/// From this width [`PaneDisplayMode::Auto`] is open, `kExpandedModeThresholdWidth`.
const EXPANDED_THRESHOLD: f32 = 1008.0;
/// Row pitch of a pane item: a 36px tile with 2px above and below.
const ITEM_HEIGHT: f32 = 40.0;
/// Height of the hamburger row, and of the top bar.
const TOP_ROW_HEIGHT: f32 = 48.0;

/// How a [`NavigationView`] lays out its pane, fluent_ui's `PaneDisplayMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneDisplayMode {
    /// Open, compact or minimal depending on the available width.
    #[default]
    Auto,
    /// A pane with icons and labels beside the content. The hamburger
    /// collapses it to compact.
    Open,
    /// A narrow strip of icons; the hamburger opens the full pane over the
    /// content.
    Compact,
    /// Only a hamburger button; the pane opens over the content.
    Minimal,
    /// Items in a row above the content.
    Top,
}

impl PaneDisplayMode {
    fn resolve(self, width: f32) -> Self {
        match self {
            Self::Auto if width >= EXPANDED_THRESHOLD => Self::Open,
            Self::Auto if width >= COMPACT_THRESHOLD => Self::Compact,
            Self::Auto => Self::Minimal,
            mode => mode,
        }
    }
}

enum PaneItemKind<T> {
    Item {
        key: T,
        icon: Option<WidgetText>,
        title: WidgetText,
        enabled: bool,
        children: Vec<PaneItem<T>>,
    },
    Header(WidgetText),
    Separator,
}

/// An entry of a [`NavigationView`] pane: a selectable item, a section
/// header or a separator. fluent_ui's `PaneItem`, `PaneItemHeader` and
/// `PaneItemSeparator`.
pub struct PaneItem<T> {
    kind: PaneItemKind<T>,
}

impl<T> PaneItem<T> {
    /// An item that selects `key` when clicked.
    pub fn new(key: T, title: impl Into<WidgetText>) -> Self {
        Self {
            kind: PaneItemKind::Item {
                key,
                icon: None,
                title: title.into(),
                enabled: true,
                children: Vec::new(),
            },
        }
    }

    pub fn header(text: impl Into<WidgetText>) -> Self {
        Self {
            kind: PaneItemKind::Header(text.into()),
        }
    }

    pub fn separator() -> Self {
        Self {
            kind: PaneItemKind::Separator,
        }
    }

    /// A 16px icon, e.g. a codepoint from [`crate::icons`] in an icon font.
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        if let PaneItemKind::Item { icon: i, .. } = &mut self.kind {
            *i = Some(icon.into());
        }
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        if let PaneItemKind::Item { enabled: e, .. } = &mut self.kind {
            *e = enabled;
        }
        self
    }

    /// Items nested under this one, fluent_ui's `PaneItemExpander`. In the
    /// open pane a click selects the item and expands or collapses them;
    /// in the compact pane and the top bar they open in a flyout.
    pub fn children(mut self, items: impl IntoIterator<Item = PaneItem<T>>) -> Self {
        if let PaneItemKind::Item { children, .. } = &mut self.kind {
            *children = items.into_iter().collect();
        }
        self
    }

    /// How many selectable items this entry holds, itself and its
    /// descendants, which is how far it moves the item index.
    fn span(&self) -> usize {
        match &self.kind {
            PaneItemKind::Item { children, .. } => 1 + keyed_count(children),
            _ => 0,
        }
    }
}

fn keyed_count<T>(items: &[PaneItem<T>]) -> usize {
    items.iter().map(PaneItem::span).sum()
}

/// Keys and titles of every selectable item, depth first, in index order.
fn collect_items<'a, T>(items: &'a [PaneItem<T>], out: &mut Vec<(&'a T, String)>) {
    for item in items {
        if let PaneItemKind::Item {
            key,
            title,
            children,
            ..
        } = &item.kind
        {
            out.push((key, title.text().to_owned()));
            collect_items(children, out);
        }
    }
}

/// Menu rows for `items`, with nested items in submenus. `index` is the
/// item index of the first entry and ends past the last.
fn menu_items<T>(
    ui: &mut Ui,
    items: &[PaneItem<T>],
    index: &mut usize,
    current: Option<usize>,
    clicked: &mut Option<usize>,
) {
    for item in items {
        match &item.kind {
            PaneItemKind::Item {
                icon,
                title,
                enabled,
                children,
                ..
            } => {
                let mine = *index;
                *index += item.span();
                if children.is_empty() {
                    let mut menu_item =
                        super::MenuItem::new(title.clone()).selected(current == Some(mine));
                    if let Some(icon) = icon {
                        menu_item = menu_item.icon(icon.clone());
                    }
                    if ui.add_enabled(*enabled, menu_item).clicked() {
                        *clicked = Some(mine);
                    }
                } else {
                    super::sub_menu(ui, title.clone(), |ui| {
                        let mut index = mine + 1;
                        menu_items(ui, children, &mut index, current, clicked);
                    });
                }
            }
            PaneItemKind::Header(text) => {
                ui.add_enabled(false, super::MenuItem::new(text.clone()));
            }
            PaneItemKind::Separator => {
                super::menu_separator(ui);
            }
        }
    }
}

/// What [`NavigationView::show`] returns.
pub struct NavigationViewResponse<R> {
    /// What the body closure returned.
    pub inner: R,
    /// Whether an item was selected this frame.
    pub changed: bool,
    /// The layout in use, with [`PaneDisplayMode::Auto`] resolved.
    pub display_mode: PaneDisplayMode,
}

#[derive(Clone, Copy, Default)]
struct NavState {
    /// Open mode: whether the pane is expanded rather than compact.
    collapsed: bool,
    /// Compact and minimal modes: whether the pane is shown over the content.
    overlay: bool,
    mode: Option<PaneDisplayMode>,
    /// The selection indicator slides from `from` to `to`, starting at `since`.
    from: Option<usize>,
    to: Option<usize>,
    since: f64,
}

/// A pane of navigation items beside or above the content, fluent_ui's
/// `NavigationView` with a `NavigationPane`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, page: &mut usize) {
/// use fluent_egui::{NavigationView, PaneItem};
///
/// let current = *page;
/// NavigationView::new("nav")
///     .title("My app")
///     .items([
///         PaneItem::new(0, "Home").icon("🏠"),
///         PaneItem::header("Library"),
///         PaneItem::new(1, "Music").icon("🎵"),
///     ])
///     .footer_items([PaneItem::new(2, "Settings").icon("⚙")])
///     .show(ui, page, |ui| {
///         ui.label(format!("Page {current}"));
///     });
/// # }
/// ```
pub struct NavigationView<T> {
    id_salt: IdSalt,
    title: Option<WidgetText>,
    items: Vec<PaneItem<T>>,
    footer_items: Vec<PaneItem<T>>,
    display_mode: PaneDisplayMode,
    open_width: f32,
    transition: Option<PageTransition>,
    search: Option<String>,
}

impl<T: PartialEq + Clone> NavigationView<T> {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            title: None,
            items: Vec::new(),
            footer_items: Vec::new(),
            display_mode: PaneDisplayMode::Auto,
            open_width: OPEN_WIDTH,
            transition: None,
            search: None,
        }
    }

    /// Shown beside the hamburger button, or before the items in top mode.
    pub fn title(mut self, title: impl Into<WidgetText>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = PaneItem<T>>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Items pinned to the bottom of the pane (the end of the top bar).
    pub fn footer_items(mut self, items: impl IntoIterator<Item = PaneItem<T>>) -> Self {
        self.footer_items = items.into_iter().collect();
        self
    }

    pub fn display_mode(mut self, mode: PaneDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    /// Width of the open pane, 320 by default.
    pub fn open_width(mut self, width: f32) -> Self {
        self.open_width = width;
        self
    }

    /// A search box in the pane that suggests item titles and selects the
    /// one picked, fluent_ui's `autoSuggestBox`. In the compact pane it is a
    /// search button that opens the full pane.
    pub fn search(mut self, placeholder: impl Into<String>) -> Self {
        self.search = Some(placeholder.into());
        self
    }

    /// How the body animates when the selection changes. Defaults to
    /// [`PageTransition::entrance`], or a horizontal slide in top mode.
    pub fn transition(mut self, transition: PageTransition) -> Self {
        self.transition = Some(transition);
        self
    }

    /// Fills the rest of `ui` with the pane and the body. `selected` is set
    /// to the key of the clicked item.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        selected: &mut T,
        add_body: impl FnOnce(&mut Ui) -> R,
    ) -> NavigationViewResponse<R> {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let ctx = ui.ctx().clone();
        let id = ui.make_persistent_id(self.id_salt);
        let rect = ui.available_rect_before_wrap();
        let mode = self.display_mode.resolve(rect.width());

        let mut state = ctx.data(|d| d.get_temp::<NavState>(id)).unwrap_or_default();
        if state.mode != Some(mode) {
            state.overlay = false;
            state.mode = Some(mode);
        }
        let now = ui.input(|i| i.time);
        let current = self.keys().into_iter().position(|k| k == selected);
        if current != state.to {
            state.from = state.to;
            state.to = current;
            state.since = now;
        }
        let slide = theme.medium_animation_duration.as_secs_f32();
        let t = ((now - state.since) as f32 / slide).clamp(0.0, 1.0);
        if t < 1.0 {
            ctx.request_repaint();
        }
        let indicator = Indicator {
            from: state.from,
            to: state.to,
            t: (theme.animation_curve)(t),
        };

        let anim = |name: &str, on: bool| {
            ctx.animate_bool_with_time_and_easing(
                id.with(name),
                on,
                theme.medium_animation_duration.as_secs_f32(),
                theme.animation_curve,
            )
        };
        let expand_t = anim("expanded", !state.collapsed);
        let overlay_t = anim("overlay", state.overlay);

        let top_offset = match mode {
            PaneDisplayMode::Top | PaneDisplayMode::Minimal => TOP_ROW_HEIGHT,
            _ => 0.0,
        };
        let inline_width = match mode {
            PaneDisplayMode::Open => egui::lerp(COMPACT_WIDTH..=self.open_width, expand_t),
            PaneDisplayMode::Compact => COMPACT_WIDTH,
            _ => 0.0,
        };
        let body_rect = Rect::from_min_max(
            pos2(rect.left() + inline_width, rect.top() + top_offset),
            rect.max,
        );

        // The body first, so an overlay pane paints over it.
        let framed = matches!(mode, PaneDisplayMode::Open | PaneDisplayMode::Compact);
        let radius = if framed {
            CornerRadius {
                nw: 8,
                ..Default::default()
            }
        } else {
            CornerRadius::ZERO
        };
        ui.painter()
            .rect_filled(body_rect, radius, theme.scaffold_background_color);
        let mut body_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(id.with("body"))
                .max_rect(body_rect),
        );
        body_ui.set_clip_rect(body_rect.intersect(ui.clip_rect()));
        let transition = self.transition.unwrap_or_else(|| {
            if mode == PaneDisplayMode::Top {
                PageTransition::horizontal_slide(false)
            } else {
                PageTransition::entrance()
            }
        });
        let inner = transition
            .show(&mut body_ui, "fluent_nav_transition", current, add_body)
            .inner;
        if framed {
            // Only the top and left edges show; the others are pushed out.
            ui.painter().rect_stroke(
                Rect::from_min_max(body_rect.min, body_rect.max + Vec2::splat(1.0)),
                radius,
                Stroke::new(1.0, r.card_stroke_color_default),
                StrokeKind::Inside,
            );
        }

        let mut clicked = None;
        let mut hamburger = false;
        match mode {
            PaneDisplayMode::Top => {
                let bar = Rect::from_min_size(rect.min, vec2(rect.width(), TOP_ROW_HEIGHT));
                clicked = self.top_bar(ui, id, bar, current, indicator);
            }
            PaneDisplayMode::Open => {
                let pane = Rect::from_min_size(rect.min, vec2(inline_width, rect.height()));
                let out = self.pane(ui, id, pane, expand_t > 0.0, current, indicator);
                clicked = out.clicked;
                if out.hamburger {
                    state.collapsed = !state.collapsed;
                }
            }
            _ => {
                if mode == PaneDisplayMode::Compact && overlay_t == 0.0 {
                    let pane = Rect::from_min_size(rect.min, vec2(COMPACT_WIDTH, rect.height()));
                    let out = self.pane(ui, id, pane, false, current, indicator);
                    clicked = out.clicked;
                    hamburger = out.hamburger;
                } else if mode == PaneDisplayMode::Minimal {
                    hamburger = self.minimal_header(ui, rect);
                }
            }
        }

        if overlay_t > 0.0 {
            let closed_width = if mode == PaneDisplayMode::Compact {
                COMPACT_WIDTH
            } else {
                0.0
            };
            let width = egui::lerp(closed_width..=self.open_width, overlay_t);
            let pane = Rect::from_min_size(rect.min, vec2(width, rect.height()));
            let out = Area::new(id.with("overlay"))
                .order(Order::Foreground)
                .fixed_pos(pane.min)
                .constrain(false)
                .show(&ctx, |ui| {
                    // Swallows clicks so they don't reach the body underneath.
                    ui.allocate_rect(pane, Sense::click());
                    let shadow = Shadow {
                        offset: [0, 8],
                        blur: 16,
                        spread: 0,
                        color: with_alpha(
                            egui::Color32::BLACK,
                            if theme.brightness.is_dark() {
                                0.26
                            } else {
                                0.14
                            },
                        ),
                    };
                    let radius = CornerRadius {
                        ne: 8,
                        se: 8,
                        ..Default::default()
                    };
                    ui.painter().add(shadow.as_shape(pane, radius));
                    ui.painter().rect(
                        pane,
                        radius,
                        r.system_fill_color_solid_neutral_background,
                        Stroke::new(1.0, r.surface_stroke_color_flyout),
                        StrokeKind::Inside,
                    );
                    self.pane(ui, id, pane, true, current, indicator)
                })
                .inner;
            if out.clicked.is_some() {
                clicked = out.clicked;
                state.overlay = false;
            }
            if out.hamburger {
                state.overlay = false;
            }
            // Light dismiss.
            let dismiss = ui.input(|i| {
                i.key_pressed(egui::Key::Escape)
                    || (i.pointer.any_pressed()
                        && i.pointer.interact_pos().is_some_and(|p| !pane.contains(p)))
            });
            if dismiss && state.overlay {
                state.overlay = false;
            }
        } else if hamburger {
            state.overlay = true;
        }

        let changed = if let Some(index) = clicked
            && Some(index) != current
            && let Some(key) = self.keys().get(index).copied()
        {
            *selected = key.clone();
            true
        } else {
            false
        };

        ctx.data_mut(|d| d.insert_temp(id, state));
        ui.advance_cursor_after_rect(rect);
        NavigationViewResponse {
            inner,
            changed,
            display_mode: mode,
        }
    }

    /// Every selectable item and its title, in index order.
    fn all_items(&self) -> Vec<(&T, String)> {
        let mut out = Vec::new();
        collect_items(&self.items, &mut out);
        collect_items(&self.footer_items, &mut out);
        out
    }

    fn keys(&self) -> Vec<&T> {
        self.all_items().into_iter().map(|(k, _)| k).collect()
    }

    /// The search box in `rect`. Returns the index of a picked item.
    fn search_box(
        &self,
        ui: &mut Ui,
        nav_id: egui::Id,
        rect: Rect,
        placeholder: &str,
    ) -> Option<usize> {
        let query_id = nav_id.with("fluent_nav_search_query");
        let mut query = ui
            .data(|d| d.get_temp::<String>(query_id))
            .unwrap_or_default();
        let titles: Vec<String> = self.all_items().into_iter().map(|(_, t)| t).collect();
        let mut child = ui.new_child(
            UiBuilder::new()
                .id_salt(query_id)
                .max_rect(rect)
                .layout(Layout::top_down(Align::Min)),
        );
        let response = super::AutoSuggestBox::new("fluent_nav_search", &mut query, &titles)
            .placeholder(placeholder)
            .width(rect.width())
            .show(&mut child);
        if response.chosen.is_some() {
            query.clear();
        }
        ui.data_mut(|d| d.insert_temp(query_id, query));
        response.chosen
    }

    /// Rows for `items` in the pane, nested ones indented under expanded
    /// parents. `inline` is false where nested items open in a flyout.
    fn pane_items(
        &self,
        ui: &mut Ui,
        items: &[PaneItem<T>],
        depth: usize,
        inline: bool,
        walk: &mut PaneWalk<'_>,
    ) {
        let (labels, current) = (walk.labels, walk.current);
        for item in items {
            match &item.kind {
                PaneItemKind::Item {
                    icon,
                    title,
                    enabled,
                    children,
                    ..
                } => {
                    let mine = walk.index;
                    walk.index += 1;
                    let nested = keyed_count(children);
                    let open = walk.expanded.contains(&mine);
                    let expander = (!children.is_empty()).then_some(open && inline);
                    let row = pane_item_row(
                        ui,
                        icon.clone(),
                        title.clone(),
                        *enabled,
                        labels,
                        current == Some(mine),
                        if inline { depth } else { 0 },
                        expander,
                    );
                    let tile = row.rect.shrink2(vec2(6.0, 2.0));
                    walk.tiles[mine] = Some(tile);
                    if children.is_empty() || inline {
                        if row.clicked() {
                            walk.clicked = Some(mine);
                            if !children.is_empty() {
                                if open {
                                    walk.expanded.retain(|i| *i != mine);
                                } else {
                                    walk.expanded.push(mine);
                                }
                            }
                        }
                    } else {
                        let theme = FluentTheme::of(ui.ctx());
                        egui::Popup::menu(&row)
                            .align(egui::RectAlign::RIGHT_START)
                            .frame(super::menu::flyout_frame(&theme))
                            .gap(4.0)
                            .width(200.0)
                            .show(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                let mut index = mine + 1;
                                menu_items(ui, children, &mut index, current, &mut walk.clicked);
                            });
                    }
                    if inline && open {
                        self.pane_items(ui, children, depth + 1, inline, walk);
                    } else {
                        // Hidden items point the indicator at their parent.
                        for tile_slot in &mut walk.tiles[mine + 1..mine + 1 + nested] {
                            *tile_slot = Some(tile);
                        }
                        walk.index += nested;
                    }
                }
                PaneItemKind::Header(text) if labels => pane_header(ui, text.clone()),
                PaneItemKind::Header(_) => {}
                PaneItemKind::Separator => pane_separator(ui),
            }
        }
    }

    /// The left pane in `rect`, with labels when `labels`.
    fn pane(
        &self,
        ui: &mut Ui,
        nav_id: egui::Id,
        rect: Rect,
        labels: bool,
        current: Option<usize>,
        indicator: Indicator,
    ) -> PaneOutput {
        let theme = FluentTheme::of(ui.ctx());
        let mut ui = ui.new_child(UiBuilder::new().max_rect(rect));
        ui.set_clip_rect(rect.intersect(ui.clip_rect()));
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        let mut out = PaneOutput {
            clicked: None,
            hamburger: hamburger_button(&mut ui, rect.min, labels.then_some(&self.title), labels),
        };

        let footer_height: f32 = if self.footer_items.is_empty() {
            0.0
        } else {
            9.0 + self
                .footer_items
                .iter()
                .map(|item| item_height(&ui, item, labels))
                .sum::<f32>()
        };
        let mut items_top = rect.top() + TOP_ROW_HEIGHT;
        if let Some(placeholder) = &self.search {
            if labels {
                let search = Rect::from_min_size(
                    pos2(rect.left() + 8.0, items_top + 4.0),
                    vec2(rect.width() - 16.0, 32.0),
                );
                if let Some(i) = self.search_box(&mut ui, nav_id, search, placeholder) {
                    out.clicked = Some(i);
                }
                items_top += 48.0;
            } else {
                let tile = Rect::from_min_size(
                    pos2(rect.left() + 6.0, items_top + 2.0),
                    vec2(COMPACT_WIDTH - 12.0, 36.0),
                );
                if search_button(&mut ui, tile).clicked() {
                    out.hamburger = true;
                }
                items_top += ITEM_HEIGHT;
            }
        }
        let items_rect = Rect::from_min_max(
            pos2(rect.left(), items_top),
            pos2(rect.right(), rect.bottom() - footer_height - 4.0),
        );
        let footer_rect = Rect::from_min_max(
            pos2(rect.left(), items_rect.bottom()),
            pos2(rect.right(), rect.bottom() - 4.0),
        );

        let expanded_id = nav_id.with("fluent_nav_expanded");
        let mut expanded = ui
            .data(|d| d.get_temp::<Vec<usize>>(expanded_id))
            .unwrap_or_default();
        let mut tiles = vec![None; keyed_count(&self.items) + keyed_count(&self.footer_items)];
        let mut walk = PaneWalk {
            labels,
            current,
            index: 0,
            tiles: &mut tiles,
            clicked: None,
            expanded: &mut expanded,
        };

        let mut items_ui = ui.new_child(UiBuilder::new().max_rect(items_rect));
        ScrollArea::vertical()
            .id_salt("fluent_nav_items")
            .auto_shrink([false; 2])
            .show(&mut items_ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                self.pane_items(ui, &self.items, 0, labels, &mut walk);
            });
        if !self.footer_items.is_empty() {
            let mut footer_ui = ui.new_child(UiBuilder::new().max_rect(footer_rect));
            footer_ui.spacing_mut().item_spacing = Vec2::ZERO;
            pane_separator(&mut footer_ui);
            self.pane_items(&mut footer_ui, &self.footer_items, 0, false, &mut walk);
        }
        if walk.clicked.is_some() {
            out.clicked = walk.clicked;
        }
        ui.data_mut(|d| d.insert_temp(expanded_id, expanded));

        // The sticky indicator: a 3x16 pill on the left of the selected tile.
        if let Some(to) = indicator.to.and_then(|i| tiles.get(i).copied().flatten()) {
            let from = indicator
                .from
                .and_then(|i| tiles.get(i).copied().flatten())
                .unwrap_or(to);
            let y = egui::lerp(from.center().y..=to.center().y, indicator.t);
            let pill = Rect::from_center_size(pos2(to.left() + 1.5, y), vec2(3.0, 16.0));
            ui.painter().rect_filled(
                pill,
                1.5,
                theme.accent_color.default_brush_for(theme.brightness),
            );
        }
        out
    }

    /// The row above the content in top mode. Items that don't fit move
    /// into a "more" menu at the end, like fluent_ui's `DynamicOverflow`.
    fn top_bar(
        &self,
        ui: &mut Ui,
        nav_id: egui::Id,
        rect: Rect,
        current: Option<usize>,
        indicator: Indicator,
    ) -> Option<usize> {
        const GAP: f32 = 4.0;
        const MORE_WIDTH: f32 = 40.0;
        let theme = FluentTheme::of(ui.ctx());
        let inner = rect.shrink2(vec2(4.0, 4.0));
        let row = |ui: &mut Ui, layout| {
            let mut ui = ui.new_child(UiBuilder::new().max_rect(inner).layout(layout));
            ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
            ui
        };
        let item_count = keyed_count(&self.items);
        let mut tiles = vec![None; item_count + keyed_count(&self.footer_items)];
        let mut clicked = None;
        let mut searched = None;
        let mut add =
            |ui: &mut Ui, item: &PaneItem<T>, index: usize, tiles: &mut Vec<Option<Rect>>| {
                match &item.kind {
                    PaneItemKind::Item {
                        icon,
                        title,
                        enabled,
                        children,
                        ..
                    } => {
                        let span = item.span();
                        let response = top_item(
                            ui,
                            icon.clone(),
                            title.clone(),
                            *enabled,
                            current.is_some_and(|c| (index..index + span).contains(&c)),
                        );
                        for tile in &mut tiles[index..index + span] {
                            *tile = Some(response.rect);
                        }
                        if children.is_empty() {
                            if response.clicked() {
                                clicked = Some(index);
                            }
                        } else {
                            super::menu::show_menu(&theme, &response, Some(200.0), |ui| {
                                let mut child_index = index + 1;
                                menu_items(ui, children, &mut child_index, current, &mut clicked);
                            });
                        }
                    }
                    PaneItemKind::Header(_) => {}
                    PaneItemKind::Separator => {
                        let (r, _) = ui.allocate_exact_size(vec2(9.0, 24.0), Sense::hover());
                        ui.painter().vline(
                            r.center().x,
                            r.y_range(),
                            Stroke::new(1.0, theme.resources.divider_stroke_color_default),
                        );
                    }
                }
            };

        // Footer items first, at the end: right to left, reversed, so they
        // keep their order.
        let mut end = row(ui, Layout::right_to_left(Align::Center));
        let mut index = tiles.len();
        for item in self.footer_items.iter().rev() {
            index -= item.span();
            add(&mut end, item, index, &mut tiles);
        }
        if let Some(placeholder) = &self.search {
            let (search, _) = end.allocate_exact_size(vec2(200.0, 32.0), Sense::hover());
            searched = self.search_box(&mut end, nav_id, search, placeholder);
        }
        let end_width = if self.footer_items.is_empty() && self.search.is_none() {
            0.0
        } else {
            end.min_rect().width() + GAP
        };

        let mut start = row(ui, Layout::left_to_right(Align::Center));
        if let Some(title) = &self.title {
            let galley = title.clone().into_galley(
                &start,
                Some(TextWrapMode::Extend),
                f32::INFINITY,
                theme.typography.body.font_id(),
            );
            let (r, _) = start.allocate_exact_size(galley.size() + vec2(24.0, 0.0), Sense::hover());
            start.painter().galley(
                pos2(r.left() + 8.0, r.center().y - galley.size().y / 2.0),
                galley,
                theme.resources.text_fill_color_primary,
            );
        }

        // How many items fit, leaving room for the "more" button if some don't.
        let widths: Vec<f32> = self
            .items
            .iter()
            .map(|item| match &item.kind {
                PaneItemKind::Item { icon, title, .. } => top_item_width(&start, icon, title) + GAP,
                PaneItemKind::Header(_) => 0.0,
                PaneItemKind::Separator => 9.0 + GAP,
            })
            .collect();
        let room = start.available_width() - end_width;
        let visible = if widths.iter().sum::<f32>() <= room {
            widths.len()
        } else {
            let mut used = MORE_WIDTH + GAP;
            widths
                .iter()
                .take_while(|w| {
                    used += *w;
                    used <= room
                })
                .count()
        };

        let mut index = 0;
        for item in &self.items[..visible] {
            add(&mut start, item, index, &mut tiles);
            index += item.span();
        }
        if visible < self.items.len() {
            let overflow_selected = current.is_some_and(|c| c >= index && c < item_count);
            let more = top_more_button(&mut start, overflow_selected);
            if overflow_selected {
                tiles[current.unwrap_or_default()] = Some(more.rect);
            }
            super::menu::show_menu(&theme, &more, Some(200.0), |ui| {
                menu_items(
                    ui,
                    &self.items[visible..],
                    &mut index,
                    current,
                    &mut clicked,
                );
            });
        }

        // The sticky indicator: a 16x3 pill under the selected item.
        if let Some(to) = indicator.to.and_then(|i| tiles.get(i).copied().flatten()) {
            let from = indicator
                .from
                .and_then(|i| tiles.get(i).copied().flatten())
                .unwrap_or(to);
            let x = egui::lerp(from.center().x..=to.center().x, indicator.t);
            let pill = Rect::from_center_size(pos2(x, to.bottom() - 2.5), vec2(16.0, 3.0));
            ui.painter().rect_filled(
                pill,
                1.5,
                theme.accent_color.default_brush_for(theme.brightness),
            );
        }
        clicked.or(searched)
    }

    /// The hamburger and title above the content in minimal mode.
    fn minimal_header(&self, ui: &mut Ui, rect: Rect) -> bool {
        let row = Rect::from_min_size(rect.min, vec2(rect.width(), TOP_ROW_HEIGHT));
        let mut ui = ui.new_child(UiBuilder::new().max_rect(row));
        hamburger_button(&mut ui, rect.min, Some(&self.title), false)
    }
}

#[derive(Clone, Copy)]
struct Indicator {
    from: Option<usize>,
    to: Option<usize>,
    t: f32,
}

/// Where [`NavigationView::pane_items`] is in the item list.
struct PaneWalk<'a> {
    labels: bool,
    current: Option<usize>,
    index: usize,
    tiles: &'a mut [Option<Rect>],
    clicked: Option<usize>,
    expanded: &'a mut Vec<usize>,
}

/// The search button of the compact pane.
fn search_button(ui: &mut Ui, tile: Rect) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let response = ui.interact(
        tile,
        ui.scope_id().with("fluent_nav_search_button"),
        Sense::click(),
    );
    let label = Localizations::of(ui.ctx()).search_label;
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let response = response.fluent_tooltip(label);
    let state = ControlState::from_response(&response);
    let fill = animate_color(
        ui.ctx(),
        response.id.with("fill"),
        subtle_fill(&theme).resolve(state),
        theme.faster_animation_duration,
    );
    ui.painter().rect_filled(tile, 4.0, fill);
    super::auto_suggest_box::paint_search_glyph(
        ui.painter(),
        tile.center(),
        theme.resources.text_fill_color_primary,
    );
    if state.focused {
        paint_focus_border(ui.painter(), tile, &theme);
    }
    response
}

#[derive(Default)]
struct PaneOutput {
    clicked: Option<usize>,
    hamburger: bool,
}

fn subtle_fill(theme: &FluentTheme) -> States<egui::Color32> {
    let r = &theme.resources;
    States::new(
        r.subtle_fill_color_transparent,
        r.subtle_fill_color_secondary,
        r.subtle_fill_color_tertiary,
        r.subtle_fill_color_transparent,
    )
}

/// The 38x36 hamburger tile at the top left of a pane, with the pane title
/// beside it when `title` is given. `expanded` picks the tooltip.
fn hamburger_button(
    ui: &mut Ui,
    origin: Pos2,
    title: Option<&Option<WidgetText>>,
    expanded: bool,
) -> bool {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let tile = Rect::from_min_size(origin + vec2(6.0, 6.0), vec2(COMPACT_WIDTH - 12.0, 36.0));
    let response = ui.interact(
        tile,
        ui.scope_id().with("fluent_nav_hamburger"),
        Sense::click(),
    );
    let l10n = Localizations::of(ui.ctx());
    let tooltip = if expanded {
        l10n.close_navigation_tooltip
    } else {
        l10n.open_navigation_tooltip
    };
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, tooltip));
    let response = response.fluent_tooltip(tooltip);
    let state = ControlState::from_response(&response);
    let fill = animate_color(
        ui.ctx(),
        response.id.with("fill"),
        subtle_fill(&theme).resolve(state),
        theme.faster_animation_duration,
    );
    let painter = ui.painter();
    painter.rect_filled(tile, 4.0, fill);
    paint_hamburger(painter, tile.center(), r.text_fill_color_primary);
    if state.focused {
        paint_focus_border(painter, tile, &theme);
    }
    if let Some(Some(title)) = title {
        let galley = title.clone().into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            f32::INFINITY,
            theme.typography.body.font_id(),
        );
        let pos = pos2(tile.right() + 8.0, tile.center().y - galley.size().y / 2.0);
        ui.painter().galley(pos, galley, r.text_fill_color_primary);
    }
    response.clicked()
}

fn item_height<T>(ui: &Ui, item: &PaneItem<T>, labels: bool) -> f32 {
    match &item.kind {
        PaneItemKind::Item { .. } => ITEM_HEIGHT,
        PaneItemKind::Header(_) if labels => header_height(ui),
        PaneItemKind::Header(_) => 0.0,
        PaneItemKind::Separator => 9.0,
    }
}

fn header_height(ui: &Ui) -> f32 {
    let theme = FluentTheme::of(ui.ctx());
    ui.fonts_mut(|f| f.row_height(&theme.typography.body_strong.font_id())) + 16.0
}

fn pane_header(ui: &mut Ui, text: WidgetText) {
    let theme = FluentTheme::of(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), header_height(ui)),
        Sense::hover(),
    );
    let galley = text.into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        rect.width() - 30.0,
        theme.typography.body_strong.font_id(),
    );
    ui.painter().galley(
        pos2(rect.left() + 18.0, rect.center().y - galley.size().y / 2.0),
        galley,
        theme.resources.text_fill_color_secondary,
    );
}

fn pane_separator(ui: &mut Ui) {
    let theme = FluentTheme::of(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 9.0), Sense::hover());
    ui.painter().hline(
        rect.x_range().shrink(6.0),
        rect.center().y,
        Stroke::new(1.0, theme.resources.divider_stroke_color_default),
    );
}

/// One item row of the left pane, fluent_ui's `PaneItem.build`.
#[allow(clippy::too_many_arguments)]
fn pane_item_row(
    ui: &mut Ui,
    icon: Option<WidgetText>,
    title: WidgetText,
    enabled: bool,
    labels: bool,
    selected: bool,
    depth: usize,
    expander: Option<bool>,
) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let label = title.text().to_owned();
    // Nested items move right by one icon column per level.
    let indent = if labels { depth as f32 * 28.0 } else { 0.0 };
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), ITEM_HEIGHT),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Tab, enabled, &label);
        info.selected = Some(selected);
        info
    });
    let response = if labels {
        response
    } else {
        response.fluent_tooltip(label)
    };
    let tile = rect.shrink2(vec2(6.0, 2.0));
    if ui.is_rect_visible(rect) {
        let mut state = ControlState::from_response(&response);
        state.disabled |= !enabled;
        // A selected tile shows one step further: hover looks pressed.
        let fills = subtle_fill(&theme);
        let fill = match (selected, state.disabled) {
            (true, false) if state.hovered => fills.pressed,
            (true, false) => fills.hovered,
            _ => fills.resolve(state),
        };
        let fill = animate_color(
            ui.ctx(),
            response.id.with("fill"),
            fill,
            theme.faster_animation_duration,
        );
        let color = state.pick(
            r.text_fill_color_primary,
            r.text_fill_color_primary,
            r.text_fill_color_secondary,
            r.text_fill_color_disabled,
        );
        let painter = ui.painter();
        painter.rect_filled(tile, 4.0, fill);
        if let Some(icon) = icon {
            let galley = icon.into_galley(
                ui,
                Some(TextWrapMode::Extend),
                f32::INFINITY,
                egui::FontId::proportional(16.0),
            );
            painter.galley(
                tile.left_center() + vec2(20.0 + indent, 0.0) - galley.size() / 2.0,
                galley,
                color,
            );
        }
        let chevron_room = if expander.is_some() { 28.0 } else { 0.0 };
        if labels {
            let galley = title.into_galley(
                ui,
                Some(TextWrapMode::Truncate),
                tile.width() - 40.0 - indent - 10.0 - chevron_room,
                theme.typography.body.font_id(),
            );
            painter
                .with_clip_rect(tile.intersect(painter.clip_rect()))
                .galley(
                    pos2(
                        tile.left() + 40.0 + indent,
                        tile.center().y - galley.size().y / 2.0,
                    ),
                    galley,
                    color,
                );
            if let Some(open) = expander {
                let at = pos2(tile.right() - 18.0, tile.center().y);
                if open {
                    crate::utils::paint_chevron_up(painter, at, color);
                } else {
                    crate::utils::paint_chevron_down(painter, at, color);
                }
            }
        } else if expander.is_some() {
            // A small corner mark: this item opens a flyout.
            crate::utils::paint_chevron_right(
                painter,
                pos2(tile.right() - 5.0, tile.bottom() - 7.0),
                color,
            );
        }
        if state.focused {
            paint_focus_border(painter, tile, &theme);
        }
    }
    response
}

fn top_item_width(ui: &Ui, icon: &Option<WidgetText>, title: &WidgetText) -> f32 {
    let theme = FluentTheme::of(ui.ctx());
    let text = title.clone().into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        theme.typography.body.font_id(),
    );
    let icon_width = if icon.is_some() { 16.0 + 12.0 } else { 0.0 };
    (12.0 + icon_width + text.size().x + 12.0).max(40.0)
}

/// The "more" button at the end of an overflowing top bar.
fn top_more_button(ui: &mut Ui, selected: bool) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let (rect, response) = ui.allocate_exact_size(vec2(40.0, ITEM_HEIGHT), Sense::click());
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Button, true, "More");
        info.selected = Some(selected);
        info
    });
    let mut state = ControlState::from_response(&response);
    state.pressed |= egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&response));
    let fill = animate_color(
        ui.ctx(),
        response.id.with("fill"),
        subtle_fill(&theme).resolve(state),
        theme.faster_animation_duration,
    );
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, fill);
    crate::utils::paint_chevron_down(painter, rect.center(), r.text_fill_color_primary);
    if state.focused {
        paint_focus_border(painter, rect, &theme);
    }
    response
}

/// One item of the top bar: icon and label, with a subtle fill on hover.
fn top_item(
    ui: &mut Ui,
    icon: Option<WidgetText>,
    title: WidgetText,
    enabled: bool,
    selected: bool,
) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let label = title.text().to_owned();
    let icon = icon.map(|i| {
        i.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            egui::FontId::proportional(16.0),
        )
    });
    let text = title.into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        theme.typography.body.font_id(),
    );
    let icon_width = if icon.is_some() { 16.0 + 12.0 } else { 0.0 };
    let width = (12.0 + icon_width + text.size().x + 12.0).max(40.0);
    let (rect, response) = ui.allocate_exact_size(
        vec2(width, ITEM_HEIGHT),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Tab, enabled, &label);
        info.selected = Some(selected);
        info
    });
    if ui.is_rect_visible(rect) {
        let mut state = ControlState::from_response(&response);
        state.disabled |= !enabled;
        let fill = animate_color(
            ui.ctx(),
            response.id.with("fill"),
            subtle_fill(&theme).resolve(state),
            theme.faster_animation_duration,
        );
        // fluent_ui's `selectedTopTextStyle` / `unselectedTopTextStyle`.
        let color = if selected {
            state.pick(
                r.text_fill_color_primary,
                r.text_fill_color_secondary,
                r.text_fill_color_tertiary,
                r.text_fill_color_disabled,
            )
        } else {
            state.pick(
                r.text_fill_color_primary,
                r.text_fill_color_primary,
                r.text_fill_color_secondary,
                r.text_fill_color_disabled,
            )
        };
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, fill);
        let mut x = rect.left() + 12.0;
        if let Some(icon) = icon {
            painter.galley(
                pos2(
                    x + 8.0 - icon.size().x / 2.0,
                    rect.center().y - icon.size().y / 2.0,
                ),
                icon,
                color,
            );
            x += icon_width;
        }
        painter.galley(pos2(x, rect.center().y - text.size().y / 2.0), text, color);
        if state.focused {
            paint_focus_border(painter, rect, &theme);
        }
    }
    response
}

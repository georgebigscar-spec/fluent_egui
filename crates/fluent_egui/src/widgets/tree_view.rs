use std::hash::{DefaultHasher, Hash, Hasher};

use egui::{
    AsIdSalt, Id, IdSalt, Rect, Role, Sense, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText, pos2,
    vec2,
};

use super::checkbox::paint_box;
use crate::FluentTheme;
use crate::utils::{
    ControlState, States, animate_color, paint_chevron_down_small, paint_chevron_right,
    paint_focus_border,
};

/// fluent_ui's `_whiteSpace`, the unit of tree indentation.
const WHITE_SPACE: f32 = 8.0;

/// How items of a [`TreeView`] can be selected, fluent_ui's
/// `TreeViewSelectionMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeViewSelectionMode {
    /// Items can be clicked, but stay unselected.
    None,
    /// One item at a time, marked with an accent pill.
    #[default]
    Single,
    /// Any number of items, with checkboxes. Checking a parent checks its
    /// whole subtree, and a partly checked parent shows a dash.
    Multiple,
}

/// A node of a [`TreeView`], fluent_ui's `TreeViewItem`.
pub struct TreeViewItem<K> {
    key: K,
    content: WidgetText,
    leading: Option<WidgetText>,
    children: Vec<TreeViewItem<K>>,
    expanded: bool,
}

impl<K> TreeViewItem<K> {
    pub fn new(key: K, content: impl Into<WidgetText>) -> Self {
        Self {
            key,
            content: content.into(),
            leading: None,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// A 20px icon before the content.
    pub fn leading(mut self, icon: impl Into<WidgetText>) -> Self {
        self.leading = Some(icon.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TreeViewItem<K>>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Whether the item starts out expanded. Afterwards the tree remembers.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// What [`TreeView::show`] returns.
pub struct TreeViewResponse<K> {
    /// The item clicked this frame, in any selection mode.
    pub clicked: Option<K>,
    /// Whether the selection changed this frame.
    pub changed: bool,
}

/// A hierarchical list with expandable items, fluent_ui's `TreeView`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, selected: &mut Vec<&'static str>) {
/// use fluent_egui::{TreeView, TreeViewItem};
///
/// let items = [TreeViewItem::new("docs", "Documents")
///     .expanded(true)
///     .children([TreeViewItem::new("cv", "CV.pdf"), TreeViewItem::new("tax", "Taxes.xlsx")])];
/// TreeView::new("files").show(ui, &items, selected);
/// # }
/// ```
pub struct TreeView {
    id_salt: IdSalt,
    selection_mode: TreeViewSelectionMode,
    narrow_spacing: bool,
}

impl TreeView {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            selection_mode: TreeViewSelectionMode::default(),
            narrow_spacing: false,
        }
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Halves the indentation and the space around icons.
    pub fn narrow_spacing(mut self, narrow: bool) -> Self {
        self.narrow_spacing = narrow;
        self
    }

    /// Shows `items`. `selected` holds the keys of the selected items: at
    /// most one in single mode, and in multiple mode every checked item,
    /// parents included once their whole subtree is checked.
    pub fn show<K: Clone + PartialEq + Hash>(
        self,
        ui: &mut Ui,
        items: &[TreeViewItem<K>],
        selected: &mut Vec<K>,
    ) -> TreeViewResponse<K> {
        let id = ui.make_persistent_id(self.id_salt);
        let mut out = TreeViewResponse {
            clicked: None,
            changed: false,
        };
        let saved_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        for item in items {
            self.item(ui, id, item, 0, selected, &mut out);
        }
        ui.spacing_mut().item_spacing = saved_spacing;
        if out.changed && self.selection_mode == TreeViewSelectionMode::Multiple {
            for item in items {
                sync_parents(item, selected);
            }
        }
        out
    }

    fn item<K: Clone + PartialEq + Hash>(
        &self,
        ui: &mut Ui,
        id: Id,
        item: &TreeViewItem<K>,
        depth: usize,
        selected: &mut Vec<K>,
        out: &mut TreeViewResponse<K>,
    ) {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let multiple = self.selection_mode == TreeViewSelectionMode::Multiple;
        let narrow = self.narrow_spacing;
        let item_id = id.with(key_hash(&item.key));
        let expanded_id = item_id.with("expanded");
        let mut expanded = ui
            .data(|d| d.get_temp::<bool>(expanded_id))
            .unwrap_or(item.expanded);
        let expandable = !item.children.is_empty();

        // A 26px tile (28 with checkboxes) with 4px/2px margins.
        let tile_height = if multiple { 28.0 } else { 26.0 };
        let (rect, _) = ui.allocate_exact_size(
            vec2(ui.available_width(), tile_height + 4.0),
            Sense::hover(),
        );
        let tile = rect.shrink2(vec2(4.0, 2.0));
        let response = ui.interact(tile, item_id, Sense::click());
        let value = if multiple && expandable {
            subtree_state(item, selected)
        } else {
            Some(selected.contains(&item.key))
        };
        let is_selected = value == Some(true);
        let label = item.content.text().to_owned();
        response.widget_info(|| {
            let mut info = WidgetInfo::labeled(Role::TreeItem, true, &label);
            if self.selection_mode != TreeViewSelectionMode::None {
                info.selected = Some(is_selected);
            }
            info
        });

        let indent = depth as f32
            * match (multiple, narrow) {
                (true, false) => WHITE_SPACE * 3.0,
                (true, true) => WHITE_SPACE * 2.0 + 4.0,
                (false, false) => WHITE_SPACE * 2.0,
                (false, true) => WHITE_SPACE,
            };
        let mut x = tile.left() + indent;
        let check_box = multiple.then(|| {
            let boxed =
                Rect::from_min_size(pos2(x + 8.0, tile.center().y - 10.0), Vec2::splat(20.0));
            x = boxed.right() + if narrow { 0.0 } else { WHITE_SPACE };
            boxed
        });
        let chevron_box = Rect::from_min_max(pos2(x, tile.top()), pos2(x + 24.0, tile.bottom()));
        x += 24.0;
        let chevron =
            expandable.then(|| ui.interact(chevron_box, item_id.with("chevron"), Sense::click()));

        if chevron.as_ref().is_some_and(|c| c.clicked()) {
            expanded = !expanded;
            ui.data_mut(|d| d.insert_temp(expanded_id, expanded));
        } else if response.clicked() {
            out.clicked = Some(item.key.clone());
            match self.selection_mode {
                TreeViewSelectionMode::None => {}
                TreeViewSelectionMode::Single => {
                    if !is_selected {
                        selected.clear();
                        selected.push(item.key.clone());
                        out.changed = true;
                    }
                }
                TreeViewSelectionMode::Multiple => {
                    set_subtree(item, selected, !is_selected);
                    out.changed = true;
                }
            }
        }

        if ui.is_rect_visible(rect) {
            let mut state = ControlState::from_response(&response);
            if let Some(chevron) = &chevron {
                let cs = ControlState::from_response(chevron);
                state.hovered |= cs.hovered;
                state.pressed |= cs.pressed;
            }
            let fills = States::new(
                r.subtle_fill_color_transparent,
                r.subtle_fill_color_secondary,
                r.subtle_fill_color_tertiary,
                r.subtle_fill_color_transparent,
            );
            // fluent_ui shows a selected item one state further along.
            let single = self.selection_mode == TreeViewSelectionMode::Single;
            let fill = match (single && is_selected, state.hovered, state.pressed) {
                (true, true, false) => fills.pressed,
                (true, _, _) => fills.hovered,
                _ => fills.resolve(state),
            };
            let fill = animate_color(
                ui.ctx(),
                item_id.with("fill"),
                fill,
                theme.faster_animation_duration,
            );
            let foreground = state.pick(
                r.text_fill_color_primary,
                r.text_fill_color_primary,
                r.text_fill_color_secondary,
                r.text_fill_color_disabled,
            );
            let painter = ui.painter();
            painter.rect_filled(tile, 6.0, fill);
            if let Some(boxed) = check_box {
                let box_state = ControlState {
                    focused: false,
                    ..state
                };
                paint_box(ui, item_id.with("check"), boxed, box_state, value);
            }
            if expandable {
                if expanded {
                    paint_chevron_down_small(painter, chevron_box.center(), foreground);
                } else {
                    paint_chevron_right(painter, chevron_box.center(), foreground);
                }
            }
            if let Some(leading) = &item.leading {
                x += if narrow { 0.0 } else { WHITE_SPACE };
                let galley = leading.clone().into_galley(
                    ui,
                    Some(TextWrapMode::Extend),
                    f32::INFINITY,
                    egui::FontId::proportional(16.0),
                );
                painter.galley(
                    pos2(
                        x + 10.0 - galley.size().x / 2.0,
                        tile.center().y - galley.size().y / 2.0,
                    ),
                    galley,
                    foreground,
                );
                x += 20.0
                    + if narrow {
                        WHITE_SPACE
                    } else {
                        2.0 * WHITE_SPACE
                    };
            } else if !narrow {
                x += WHITE_SPACE;
            }
            let font = egui::FontId::new(12.0, theme.typography.body.font_id().family);
            let galley = item.content.clone().into_galley(
                ui,
                Some(TextWrapMode::Truncate),
                (tile.right() - x - 4.0).max(0.0),
                font,
            );
            painter.galley(
                pos2(x, tile.center().y - galley.size().y / 2.0),
                galley,
                foreground,
            );
            if single && is_selected {
                let pill = Rect::from_min_max(
                    pos2(tile.left(), tile.top() + 6.0),
                    pos2(tile.left() + 3.0, tile.bottom() - 6.0),
                );
                painter.rect_filled(
                    pill,
                    1.5,
                    theme.accent_color.default_brush_for(theme.brightness),
                );
            }
            if state.focused {
                paint_focus_border(painter, tile, &theme);
            }
        }

        if expandable && expanded {
            for child in &item.children {
                self.item(ui, id, child, depth + 1, selected, out);
            }
        }
    }
}

/// Checked when every leaf below is selected, unchecked when none is, and
/// indeterminate (`None`) otherwise.
fn subtree_state<K: PartialEq>(item: &TreeViewItem<K>, selected: &[K]) -> Option<bool> {
    let (mut any, mut all) = (false, true);
    visit_leaves(item, &mut |leaf| {
        let on = selected.contains(&leaf.key);
        any |= on;
        all &= on;
    });
    match (any, all) {
        (_, true) => Some(true),
        (false, _) => Some(false),
        _ => None,
    }
}

fn visit_leaves<K>(item: &TreeViewItem<K>, f: &mut impl FnMut(&TreeViewItem<K>)) {
    if item.children.is_empty() {
        f(item);
    }
    for child in &item.children {
        visit_leaves(child, f);
    }
}

fn set_subtree<K: Clone + PartialEq>(item: &TreeViewItem<K>, selected: &mut Vec<K>, on: bool) {
    selected.retain(|k| *k != item.key);
    if on {
        selected.push(item.key.clone());
    }
    for child in &item.children {
        set_subtree(child, selected, on);
    }
}

/// Puts each parent in `selected` exactly when its whole subtree is.
fn sync_parents<K: Clone + PartialEq>(item: &TreeViewItem<K>, selected: &mut Vec<K>) {
    if item.children.is_empty() {
        return;
    }
    for child in &item.children {
        sync_parents(child, selected);
    }
    let all = subtree_state(item, selected) == Some(true);
    selected.retain(|k| *k != item.key);
    if all {
        selected.push(item.key.clone());
    }
}

/// Keys only need `Hash`, so hash them into an id salt ourselves.
fn key_hash(key: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

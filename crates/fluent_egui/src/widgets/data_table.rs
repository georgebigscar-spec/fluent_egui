//! A data table in the spirit of TanStack Table and shadcn/ui's data table,
//! drawn with Fluent resources. fluent_ui has no counterpart.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};

use egui::{
    Align, Align2, AsIdSalt, Context, CursorIcon, Frame, Id, LayerId, Layout, Order, Painter,
    Popup, PopupCloseBehavior, Pos2, Rect, RectAlign, Response, ScrollArea, Sense, Stroke,
    TextWrapMode, Ui, UiBuilder, Vec2, WidgetText, pos2, vec2,
};

use super::button::{Button, ButtonStyle};
use super::checkbox::paint_box;
use super::glyph_button;
use super::menu::{ComboBox, MenuItem, combo_value, flyout_frame, menu_separator};
use super::text_box::TextBox;
use super::tooltip::FluentTooltip as _;
use crate::utils::{
    ControlState, States, animate_color, paint_chevron_down, paint_chevron_up, with_alpha,
};
use crate::{FluentTheme, Localizations};

const HEADER_HEIGHT: f32 = 36.0;
const HANDLE_WIDTH: f32 = 28.0;
const CHECK_WIDTH: f32 = 44.0;
const ACTIONS_WIDTH: f32 = 48.0;
const CELL_PADDING: f32 = 12.0;

type CellUi<'a, T> = Box<dyn Fn(&mut Ui, &T) + 'a>;
type Compare<'a, T> = Box<dyn Fn(&T, &T) -> Ordering + 'a>;
type RowActions<'a, T, A> = Box<dyn FnMut(&mut Ui, &T) -> Option<A> + 'a>;
type RowKey<'a, T> = Box<dyn Fn(&T) -> Id + 'a>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// How a column can be filtered from the toolbar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColumnFilter {
    #[default]
    None,
    /// A text box; rows whose cell contains the text (ignoring case) pass.
    Text,
    /// A button with a list of the column's values and their counts; rows
    /// whose value is checked pass. Nothing checked means no filter.
    Faceted,
}

/// How rows of a [`DataTable`] can be selected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataTableSelectionMode {
    None,
    /// Clicking a row selects it, and only it.
    Single,
    /// Checkboxes on every row and a select-all box in the header. Clicking
    /// a row toggles it too.
    #[default]
    Multiple,
}

/// One column of a [`DataTable`], TanStack's `ColumnDef`: what to show, how
/// to format, sort and filter it.
pub struct Column<'a, T> {
    id: String,
    header: String,
    width: f32,
    min_width: f32,
    resizable: bool,
    sortable: bool,
    hideable: bool,
    searchable: bool,
    align: Align,
    filter: ColumnFilter,
    facet_options: Option<Vec<String>>,
    text: Box<dyn Fn(&T) -> String + 'a>,
    cell_ui: Option<CellUi<'a, T>>,
    compare: Option<Compare<'a, T>>,
}

impl<'a, T> Column<'a, T> {
    /// A column called `id` (unique within the table) with the title
    /// `header`. `text` formats a row's cell; it is also what search, text
    /// filters, facets and (by default) sorting look at.
    pub fn new(
        id: impl Into<String>,
        header: impl Into<String>,
        text: impl Fn(&T) -> String + 'a,
    ) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width: 160.0,
            min_width: 48.0,
            resizable: true,
            sortable: true,
            hideable: true,
            searchable: true,
            align: Align::Min,
            filter: ColumnFilter::None,
            facet_options: None,
            text: Box::new(text),
            cell_ui: None,
            compare: None,
        }
    }

    /// Initial width, 160 by default. The user can resize it afterwards.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Smallest width the user can drag the column to, 48 by default.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Whether the column is listed in the "Columns" menu, so it can be
    /// hidden. True by default.
    pub fn hideable(mut self, hideable: bool) -> Self {
        self.hideable = hideable;
        self
    }

    /// Whether the table's search box looks at this column. True by default.
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    /// Horizontal alignment of the cells and header. `Align::Max` suits
    /// numbers.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sorts with `compare` instead of comparing the cell text (numbers in
    /// the text compare as numbers).
    pub fn sort_by(mut self, compare: impl Fn(&T, &T) -> Ordering + 'a) -> Self {
        self.compare = Some(Box::new(compare));
        self
    }

    /// Sorts by the key `key` returns.
    pub fn sort_key<K: Ord>(self, key: impl Fn(&T) -> K + 'a) -> Self {
        self.sort_by(move |a, b| key(a).cmp(&key(b)))
    }

    /// Adds a text box filter for this column to the toolbar.
    pub fn text_filter(mut self) -> Self {
        self.filter = ColumnFilter::Text;
        self
    }

    /// Adds a faceted filter for this column to the toolbar, listing the
    /// distinct cell texts.
    pub fn faceted(mut self) -> Self {
        self.filter = ColumnFilter::Faceted;
        self
    }

    /// A faceted filter listing `options` in this order, rather than the
    /// distinct values sorted.
    pub fn facet_options(mut self, options: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.filter = ColumnFilter::Faceted;
        self.facet_options = Some(options.into_iter().map(Into::into).collect());
        self
    }

    /// Draws cells with `cell` instead of painting the text. The cell's `Ui`
    /// is laid out left to right, centered vertically and clipped to the
    /// cell.
    pub fn cell(mut self, cell: impl Fn(&mut Ui, &T) + 'a) -> Self {
        self.cell_ui = Some(Box::new(cell));
        self
    }

    fn compare(&self, a: &T, b: &T) -> Ordering {
        match &self.compare {
            Some(compare) => compare(a, b),
            None => compare_text(&(self.text)(a), &(self.text)(b)),
        }
    }
}

/// Compares numbers as numbers and anything else as text, ignoring case.
fn compare_text(a: &str, b: &str) -> Ordering {
    let number = |s: &str| {
        s.trim()
            .trim_start_matches(['$', '€', '£'])
            .replace([',', ' '], "")
            .parse::<f64>()
            .ok()
    };
    if let (Some(x), Some(y)) = (number(a), number(b)) {
        return x.partial_cmp(&y).unwrap_or(Ordering::Equal);
    }
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| a.cmp(b))
}

/// The texts a [`DataTable`] shows. Picked from the installed
/// [`Localizations`] (English or Russian) unless set with
/// [`DataTable::labels`].
#[derive(Clone, Copy, Debug)]
pub struct DataTableLabels {
    pub search: &'static str,
    /// Placeholder of a column's text filter, given the column header.
    pub filter: fn(&str) -> String,
    pub reset: &'static str,
    pub columns: &'static str,
    pub no_results: &'static str,
    pub rows_per_page: &'static str,
    pub clear_filters: &'static str,
    pub open_menu: &'static str,
    pub drag_to_reorder: &'static str,
    pub reorder_needs_unsorted: &'static str,
    pub sort_hint: &'static str,
    /// "Page 1 of 10".
    pub page_of: fn(usize, usize) -> String,
    /// "2 of 100 row(s) selected."
    pub selected_of: fn(usize, usize) -> String,
    /// "2 selected", on a faceted filter button.
    pub selected_count: fn(usize) -> String,
}

impl DataTableLabels {
    pub const EN: Self = Self {
        search: "Search…",
        filter: |header| format!("Filter {}…", header.to_lowercase()),
        reset: "Reset",
        columns: "Columns",
        no_results: "No results.",
        rows_per_page: "Rows per page",
        clear_filters: "Clear filters",
        open_menu: "Open menu",
        drag_to_reorder: "Drag to reorder",
        reorder_needs_unsorted: "Clear sorting to reorder rows",
        sort_hint: "Click to sort, Shift+click to sort by several columns, drag to move",
        page_of: |page, pages| format!("Page {page} of {pages}"),
        selected_of: |selected, total| format!("{selected} of {total} row(s) selected."),
        selected_count: |n| format!("{n} selected"),
    };

    pub const RU: Self = Self {
        search: "Поиск…",
        filter: |header| format!("Фильтр: {}…", header.to_lowercase()),
        reset: "Сбросить",
        columns: "Столбцы",
        no_results: "Нет результатов.",
        rows_per_page: "Строк на странице",
        clear_filters: "Очистить фильтры",
        open_menu: "Открыть меню",
        drag_to_reorder: "Перетащите, чтобы переместить",
        reorder_needs_unsorted: "Отключите сортировку, чтобы перемещать строки",
        sort_hint: "Щелчок — сортировка, Shift+щелчок — по нескольким столбцам, перетаскивание — перемещение",
        page_of: |page, pages| format!("Страница {page} из {pages}"),
        selected_of: |selected, total| format!("Выбрано строк: {selected} из {total}."),
        selected_count: |n| format!("выбрано: {n}"),
    };

    /// Russian for `ru` locales, English otherwise.
    pub fn for_locale(locale: &str) -> Self {
        if locale == "ru" || locale.starts_with("ru_") {
            Self::RU
        } else {
            Self::EN
        }
    }
}

/// Everything the user can change in a [`DataTable`], TanStack's table
/// state. The table keeps it in egui memory under [`DataTableResponse::id`];
/// read or change it with [`DataTableState::load`] and
/// [`DataTableState::store`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataTableState {
    /// Column ids and directions, the first one sorting first.
    pub sorting: Vec<(String, SortDirection)>,
    /// The search box text.
    pub search: String,
    /// Text filters by column id.
    pub text_filters: BTreeMap<String, String>,
    /// Checked facet values by column id.
    pub facet_filters: BTreeMap<String, BTreeSet<String>>,
    /// Zero-based page index.
    pub page: usize,
    pub page_size: usize,
    /// Keys of the selected rows, see [`DataTable::row_key`].
    pub selected: HashSet<Id>,
    /// Ids of the hidden columns.
    pub hidden: BTreeSet<String>,
    /// Column ids in display order.
    pub order: Vec<String>,
    /// Column widths the user dragged, by column id.
    pub widths: BTreeMap<String, f32>,
}

impl DataTableState {
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data(|d| d.get_temp(id))
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    /// Whether the search box or any column filter is in use.
    pub fn is_filtered(&self) -> bool {
        !self.search.is_empty()
            || self.text_filters.values().any(|t| !t.is_empty())
            || self.facet_filters.values().any(|s| !s.is_empty())
    }

    pub fn clear_filters(&mut self) {
        self.search.clear();
        self.text_filters.clear();
        self.facet_filters.clear();
    }

    /// Where `column` is in the sort order, and its direction.
    pub fn sort_of(&self, column: &str) -> Option<(usize, SortDirection)> {
        self.sorting
            .iter()
            .position(|(c, _)| c == column)
            .map(|i| (i, self.sorting[i].1))
    }

    /// A header click: ascending, descending, then unsorted. With `multi`
    /// (Shift held) the other sorted columns stay, and this one is added
    /// after them.
    pub fn toggle_sort(&mut self, column: &str, multi: bool) {
        let current = self.sort_of(column);
        let next = match current {
            None => Some(SortDirection::Ascending),
            Some((_, SortDirection::Ascending)) => Some(SortDirection::Descending),
            Some((_, SortDirection::Descending)) => None,
        };
        if multi {
            match (current, next) {
                (Some((i, _)), Some(dir)) => self.sorting[i].1 = dir,
                (Some((i, _)), None) => {
                    self.sorting.remove(i);
                }
                (None, Some(dir)) => self.sorting.push((column.to_owned(), dir)),
                (None, None) => {}
            }
        } else {
            self.sorting.clear();
            if let Some(dir) = next {
                self.sorting.push((column.to_owned(), dir));
            }
        }
    }

    /// Lists every column in `order` once, keeping the user's order.
    fn sync_columns<T>(&mut self, columns: &[Column<'_, T>]) {
        self.order.retain(|id| columns.iter().any(|c| &c.id == id));
        for column in columns {
            if !self.order.contains(&column.id) {
                self.order.push(column.id.clone());
            }
        }
    }

    fn filter_key(
        &self,
    ) -> (
        String,
        BTreeMap<String, String>,
        BTreeMap<String, BTreeSet<String>>,
    ) {
        (
            self.search.clone(),
            self.text_filters.clone(),
            self.facet_filters.clone(),
        )
    }
}

/// The indices of `rows` that pass the search and filters of `state`. The
/// facet filter of `skip_facet` is ignored, for counting that facet's values.
fn filter_rows<T>(
    columns: &[Column<'_, T>],
    rows: &[T],
    state: &DataTableState,
    skip_facet: Option<&str>,
) -> Vec<usize> {
    let search = state.search.trim().to_lowercase();
    let searched: Vec<&Column<'_, T>> = columns
        .iter()
        .filter(|c| c.searchable && !state.hidden.contains(&c.id))
        .collect();
    let texts: Vec<(&Column<'_, T>, String)> = columns
        .iter()
        .filter_map(|c| {
            let text = state.text_filters.get(&c.id)?.trim().to_lowercase();
            (c.filter == ColumnFilter::Text && !text.is_empty()).then_some((c, text))
        })
        .collect();
    let facets: Vec<(&Column<'_, T>, &BTreeSet<String>)> = columns
        .iter()
        .filter(|c| Some(c.id.as_str()) != skip_facet)
        .filter_map(|c| {
            let values = state.facet_filters.get(&c.id)?;
            (c.filter == ColumnFilter::Faceted && !values.is_empty()).then_some((c, values))
        })
        .collect();
    (0..rows.len())
        .filter(|&i| {
            let row = &rows[i];
            (search.is_empty()
                || searched
                    .iter()
                    .any(|c| (c.text)(row).to_lowercase().contains(&search)))
                && texts
                    .iter()
                    .all(|(c, text)| (c.text)(row).to_lowercase().contains(text))
                && facets
                    .iter()
                    .all(|(c, values)| values.contains(&(c.text)(row)))
        })
        .collect()
}

/// Stable multi-column sort of `indices` into `rows`.
fn sort_rows<T>(
    columns: &[Column<'_, T>],
    rows: &[T],
    indices: &mut [usize],
    sorting: &[(String, SortDirection)],
) {
    let keys: Vec<(&Column<'_, T>, SortDirection)> = sorting
        .iter()
        .filter_map(|(id, dir)| Some((columns.iter().find(|c| &c.id == id)?, *dir)))
        .collect();
    if keys.is_empty() {
        return;
    }
    indices.sort_by(|&a, &b| {
        for (column, dir) in &keys {
            let ord = column.compare(&rows[a], &rows[b]);
            let ord = match dir {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            };
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    });
}

/// Moves `v[from]` so it lands just before what was `v[before]`, or at the
/// end when `before == v.len()`.
fn move_item<T>(v: &mut Vec<T>, from: usize, before: usize) {
    if from >= v.len() || before > v.len() || before == from || before == from + 1 {
        return;
    }
    let item = v.remove(from);
    let to = if before > from { before - 1 } else { before };
    v.insert(to, item);
}

/// What is being dragged, kept in memory between frames.
#[derive(Clone, Debug, PartialEq)]
enum Drag {
    Row(Id),
    Column(String),
}

/// What [`DataTable::show`] returns.
pub struct DataTableResponse<A> {
    pub response: Response,
    /// Where the table keeps its [`DataTableState`].
    pub id: Id,
    /// The row whose action menu returned something, as an index into the
    /// rows, and what it returned.
    pub action: Option<(usize, A)>,
    /// Indices of the selected rows, in row order.
    pub selected: Vec<usize>,
    pub selection_changed: bool,
    /// The row clicked this frame.
    pub clicked_row: Option<usize>,
    /// Whether rows were dragged into a new order this frame.
    pub rows_reordered: bool,
    /// How many rows pass the search and filters.
    pub filtered_rows: usize,
}

/// A table with sorting, filtering, pagination, row selection, row actions,
/// column visibility, resizing and drag-and-drop reordering, after TanStack
/// Table and shadcn/ui's data table.
///
/// The table owns its UI state (see [`DataTableState`]); the rows stay
/// yours. They are taken as `&mut Vec` so that dragging can reorder them.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// use fluent_egui::{Column, DataTable, MenuItem};
///
/// struct User { id: u32, name: String, role: &'static str, age: u32 }
/// let mut users = vec![User { id: 1, name: "Ada".into(), role: "Admin", age: 36 }];
///
/// let response = DataTable::new("users")
///     .row_key(|u: &User| u.id)
///     .column(Column::new("name", "Name", |u: &User| u.name.clone()).text_filter())
///     .column(Column::new("role", "Role", |u: &User| u.role.to_owned()).faceted())
///     .column(Column::new("age", "Age", |u: &User| u.age.to_string()).sort_key(|u: &User| u.age))
///     .row_actions(|ui, _user| {
///         if ui.add(MenuItem::new("Delete")).clicked() {
///             return Some("delete");
///         }
///         None
///     })
///     .show(ui, &mut users);
/// if let Some((row, "delete")) = response.action {
///     users.remove(row);
/// }
/// # }
/// ```
pub struct DataTable<'a, T, A = ()> {
    config: Config<'a, T>,
    actions: Option<RowActions<'a, T, A>>,
}

struct Config<'a, T> {
    id: Id,
    columns: Vec<Column<'a, T>>,
    row_key: Option<RowKey<'a, T>>,
    selection: DataTableSelectionMode,
    search: bool,
    column_picker: bool,
    pagination: bool,
    page_sizes: Vec<usize>,
    page_size: usize,
    row_reorder: bool,
    column_reorder: bool,
    max_height: f32,
    row_height: f32,
    striped: bool,
    labels: Option<DataTableLabels>,
    default_sort: Vec<(String, SortDirection)>,
    default_hidden: Vec<String>,
}

impl<'a, T> DataTable<'a, T, ()> {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            config: Config {
                id: Id::unique("fluent_egui::DataTable").with(id_salt),
                columns: Vec::new(),
                row_key: None,
                selection: DataTableSelectionMode::Multiple,
                search: true,
                column_picker: true,
                pagination: true,
                page_sizes: vec![10, 20, 30, 50, 100],
                page_size: 10,
                row_reorder: false,
                column_reorder: true,
                max_height: 440.0,
                row_height: 40.0,
                striped: false,
                labels: None,
                default_sort: Vec::new(),
                default_hidden: Vec::new(),
            },
            actions: None,
        }
    }
}

impl<'a, T, A> DataTable<'a, T, A> {
    pub fn column(mut self, column: Column<'a, T>) -> Self {
        self.config.columns.push(column);
        self
    }

    pub fn columns(mut self, columns: impl IntoIterator<Item = Column<'a, T>>) -> Self {
        self.config.columns.extend(columns);
        self
    }

    /// A stable key for each row, so selection follows rows when they are
    /// sorted, reordered, inserted or removed. Without it rows are keyed by
    /// index.
    pub fn row_key<K: AsIdSalt>(mut self, key: impl Fn(&T) -> K + 'a) -> Self {
        self.config.row_key = Some(Box::new(move |row| Id::NULL.with(key(row))));
        self
    }

    pub fn selection_mode(mut self, mode: DataTableSelectionMode) -> Self {
        self.config.selection = mode;
        self
    }

    /// A "⋯" button at the end of each row that opens a menu filled by
    /// `actions`. What it returns for a row ends up in
    /// [`DataTableResponse::action`].
    pub fn row_actions<B>(
        self,
        actions: impl FnMut(&mut Ui, &T) -> Option<B> + 'a,
    ) -> DataTable<'a, T, B> {
        DataTable {
            config: self.config,
            actions: Some(Box::new(actions)),
        }
    }

    /// Whether the toolbar has a search box over all searchable columns.
    pub fn search(mut self, search: bool) -> Self {
        self.config.search = search;
        self
    }

    /// Whether the toolbar has the "Columns" menu to hide and show columns.
    pub fn column_picker(mut self, column_picker: bool) -> Self {
        self.config.column_picker = column_picker;
        self
    }

    /// Splits the rows into pages. On by default; off shows every row in
    /// the scrolling body.
    pub fn pagination(mut self, pagination: bool) -> Self {
        self.config.pagination = pagination;
        self
    }

    /// Initial rows per page, 10 by default.
    pub fn page_size(mut self, page_size: usize) -> Self {
        self.config.page_size = page_size.max(1);
        self
    }

    /// The choices of the rows-per-page box.
    pub fn page_sizes(mut self, sizes: impl IntoIterator<Item = usize>) -> Self {
        self.config.page_sizes = sizes.into_iter().filter(|&s| s > 0).collect();
        self
    }

    /// Adds a drag handle to each row. Dragging moves the row in the `Vec`;
    /// it is disabled while the table is sorted.
    pub fn reorderable_rows(mut self, reorderable: bool) -> Self {
        self.config.row_reorder = reorderable;
        self
    }

    /// Whether headers can be dragged to reorder columns. True by default.
    pub fn reorderable_columns(mut self, reorderable: bool) -> Self {
        self.config.column_reorder = reorderable;
        self
    }

    /// Height of the scrolling body before it scrolls, 440 by default.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.config.max_height = max_height;
        self
    }

    pub fn row_height(mut self, row_height: f32) -> Self {
        self.config.row_height = row_height;
        self
    }

    /// Shades every other row.
    pub fn striped(mut self, striped: bool) -> Self {
        self.config.striped = striped;
        self
    }

    pub fn labels(mut self, labels: DataTableLabels) -> Self {
        self.config.labels = Some(labels);
        self
    }

    /// Sorting before the user changes it.
    pub fn default_sort(mut self, column: impl Into<String>, direction: SortDirection) -> Self {
        self.config.default_sort.push((column.into(), direction));
        self
    }

    /// Columns hidden before the user changes it.
    pub fn default_hidden(mut self, column: impl Into<String>) -> Self {
        self.config.default_hidden.push(column.into());
        self
    }

    pub fn show(self, ui: &mut Ui, rows: &mut Vec<T>) -> DataTableResponse<A> {
        let Self {
            config: cfg,
            mut actions,
        } = self;
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let labels = cfg
            .labels
            .unwrap_or_else(|| DataTableLabels::for_locale(Localizations::of(ui.ctx()).locale));
        let id = cfg.id;
        let mut state = DataTableState::load(ui.ctx(), id).unwrap_or_else(|| DataTableState {
            sorting: cfg.default_sort.clone(),
            page_size: cfg.page_size,
            hidden: cfg.default_hidden.iter().cloned().collect(),
            ..Default::default()
        });
        state.sync_columns(&cfg.columns);
        let selected_before = state.selected.clone();
        let filters_before = state.filter_key();
        let multi = cfg.selection == DataTableSelectionMode::Multiple;

        let keys: Vec<Id> = rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                cfg.row_key
                    .as_ref()
                    .map_or_else(|| Id::NULL.with(i), |key| key(row))
            })
            .collect();

        // Toolbar: search, column filters and reset on the left, wrapping;
        // the column picker on the right.
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            // Room for the picker: its text, padding and chevron.
            let picker = if cfg.column_picker {
                let text = ui.painter().layout_no_wrap(
                    labels.columns.to_owned(),
                    theme.typography.body.font_id(),
                    r.text_fill_color_primary,
                );
                text.size().x + 64.0
            } else {
                0.0
            };
            ui.allocate_ui_with_layout(
                vec2((ui.available_width() - picker).max(0.0), 32.0),
                Layout::left_to_right(Align::Center).with_main_wrap(true),
                |ui| {
                    if cfg.search {
                        TextBox::new(&mut state.search)
                            .placeholder(labels.search)
                            .width(200.0)
                            .id_salt(id.with("search"))
                            .clear_button(true)
                            .show(ui);
                    }
                    for column in &cfg.columns {
                        if column.filter == ColumnFilter::Text {
                            let text = state.text_filters.entry(column.id.clone()).or_default();
                            TextBox::new(text)
                                .placeholder((labels.filter)(&column.header))
                                .width(160.0)
                                .id_salt(id.with(("filter", &column.id)))
                                .clear_button(true)
                                .show(ui);
                        }
                    }
                    for column in &cfg.columns {
                        if column.filter == ColumnFilter::Faceted {
                            facet_filter(
                                ui,
                                &theme,
                                &labels,
                                column,
                                &cfg.columns,
                                rows,
                                &mut state,
                            );
                        }
                    }
                    if state.is_filtered()
                        && ui
                            .add(
                                Button::new(format!("{}  ✕", labels.reset))
                                    .style(subtle_button(&theme)),
                            )
                            .clicked()
                    {
                        state.clear_filters();
                    }
                },
            );
            if cfg.column_picker {
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    column_picker(ui, &theme, &labels, &cfg.columns, &mut state);
                });
            }
        });
        ui.add_space(8.0);
        if state.filter_key() != filters_before {
            state.page = 0;
        }

        // The row model: filter, sort, paginate.
        let mut filtered = filter_rows(&cfg.columns, rows, &state, None);
        sort_rows(&cfg.columns, rows, &mut filtered, &state.sorting);
        let page_size = state.page_size.max(1);
        let page_count = if cfg.pagination {
            filtered.len().div_ceil(page_size).max(1)
        } else {
            1
        };
        state.page = state.page.min(page_count - 1);
        let page_rows: Vec<usize> = if cfg.pagination {
            filtered
                .iter()
                .copied()
                .skip(state.page * page_size)
                .take(page_size)
                .collect()
        } else {
            filtered.clone()
        };

        let visible: Vec<usize> = state
            .order
            .iter()
            .filter(|cid| !state.hidden.contains(*cid))
            .filter_map(|cid| cfg.columns.iter().position(|c| &c.id == cid))
            .collect();
        let mut widths: Vec<f32> = visible
            .iter()
            .map(|&ci| {
                let c = &cfg.columns[ci];
                state
                    .widths
                    .get(&c.id)
                    .copied()
                    .unwrap_or(c.width)
                    .max(c.min_width)
            })
            .collect();
        let lead = if cfg.row_reorder { HANDLE_WIDTH } else { 0.0 }
            + if multi { CHECK_WIDTH } else { 0.0 };
        let trail = if actions.is_some() {
            ACTIONS_WIDTH
        } else {
            0.0
        };

        let mut drag: Option<Drag> = ui.data(|d| d.get_temp(id.with("drag")));
        let pointer = ui.input(|i| i.pointer.latest_pos());
        let released = ui.input(|i| !i.pointer.any_down());
        let reorder_rows = cfg.row_reorder && state.sorting.is_empty();
        let mut action = None;
        let mut clicked_row = None;
        let mut rows_reordered = false;
        let mut row_rects: Vec<(usize, Rect)> = Vec::new();
        let mut header_rects: Vec<(usize, Rect)> = Vec::new();
        let mut table_rect = Rect::NOTHING;

        let frame = Frame::new()
            .fill(r.card_background_fill_color_default)
            .stroke(Stroke::new(1.0, r.card_stroke_color_default))
            .corner_radius(8)
            .inner_margin(1);
        let response = frame
            .show(ui, |ui| {
                let row_w = (lead + widths.iter().sum::<f32>() + trail).max(ui.available_width());
                ScrollArea::horizontal()
                    .id_salt(id.with("hscroll"))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;

                        // Header.
                        let (header, _) =
                            ui.allocate_exact_size(vec2(row_w, HEADER_HEIGHT), Sense::hover());
                        let painter = ui.painter().clone();
                        let mut x =
                            header.left() + if cfg.row_reorder { HANDLE_WIDTH } else { 0.0 };
                        if multi {
                            let cell = Rect::from_min_size(
                                pos2(x, header.top()),
                                vec2(CHECK_WIDTH, HEADER_HEIGHT),
                            );
                            let on_page = page_rows
                                .iter()
                                .filter(|&&i| state.selected.contains(&keys[i]))
                                .count();
                            let value = if on_page == 0 {
                                Some(false)
                            } else if on_page == page_rows.len() {
                                Some(true)
                            } else {
                                None
                            };
                            if checkbox(ui, id.with("select_all"), cell, value).clicked() {
                                for &i in &page_rows {
                                    if value == Some(true) {
                                        state.selected.remove(&keys[i]);
                                    } else {
                                        state.selected.insert(keys[i]);
                                    }
                                }
                            }
                            x += CHECK_WIDTH;
                        }
                        for (vi, &ci) in visible.iter().enumerate() {
                            let column = &cfg.columns[ci];
                            let cell = Rect::from_min_size(
                                pos2(x, header.top()),
                                vec2(widths[vi], HEADER_HEIGHT),
                            );
                            x += widths[vi];
                            header_rects.push((ci, cell));
                            let sense = match (column.sortable, cfg.column_reorder) {
                                (true, true) => Sense::click_and_drag(),
                                (true, false) => Sense::click(),
                                (false, true) => Sense::drag(),
                                (false, false) => Sense::hover(),
                            };
                            let mut resp =
                                ui.interact(cell, id.with(("header", &column.id)), sense);
                            if column.sortable || cfg.column_reorder {
                                resp = resp.fluent_tooltip(labels.sort_hint);
                            }
                            if resp.clicked() && column.sortable {
                                let shift = ui.input(|i| i.modifiers.shift);
                                state.toggle_sort(&column.id, shift);
                            }
                            if resp.drag_started() && cfg.column_reorder {
                                drag = Some(Drag::Column(column.id.clone()));
                            }
                            let dragging = drag == Some(Drag::Column(column.id.clone()));
                            if resp.hovered() && column.sortable || dragging {
                                ui.ctx().set_cursor_icon(if dragging {
                                    CursorIcon::Grabbing
                                } else {
                                    CursorIcon::PointingHand
                                });
                            }
                            let fill = if dragging {
                                r.subtle_fill_color_tertiary
                            } else if resp.hovered() && column.sortable {
                                r.subtle_fill_color_secondary
                            } else {
                                r.subtle_fill_color_transparent
                            };
                            let fill = animate_color(
                                ui.ctx(),
                                resp.id.with("fill"),
                                fill,
                                theme.faster_animation_duration,
                            );
                            painter.rect_filled(cell.shrink(2.0), 4.0, fill);
                            paint_header(
                                ui,
                                &painter,
                                &theme,
                                column,
                                cell,
                                state.sort_of(&column.id),
                                state.sorting.len() > 1,
                                resp.hovered(),
                            );
                        }
                        // Resize handles go on top of the headers next to them.
                        for (vi, &(ci, cell)) in header_rects.iter().enumerate() {
                            let column = &cfg.columns[ci];
                            let line_color = if column.resizable {
                                let handle = Rect::from_x_y_ranges(
                                    cell.right() - 4.0..=cell.right() + 4.0,
                                    cell.y_range(),
                                );
                                let resp = ui.interact(
                                    handle,
                                    id.with(("resize", &column.id)),
                                    Sense::drag(),
                                );
                                if resp.hovered() || resp.dragged() {
                                    ui.ctx().set_cursor_icon(CursorIcon::ResizeColumn);
                                }
                                if resp.dragged() {
                                    let w =
                                        (widths[vi] + resp.drag_delta().x).max(column.min_width);
                                    widths[vi] = w;
                                    state.widths.insert(column.id.clone(), w);
                                }
                                if resp.double_clicked() {
                                    state.widths.remove(&column.id);
                                }
                                if resp.hovered() || resp.dragged() {
                                    Some(theme.accent_color.default_brush_for(theme.brightness))
                                } else {
                                    Some(r.divider_stroke_color_default)
                                }
                            } else {
                                (vi + 1 < header_rects.len())
                                    .then_some(r.divider_stroke_color_default)
                            };
                            if let Some(color) = line_color {
                                painter.vline(
                                    cell.right() - 0.5,
                                    (cell.top() + 10.0)..=(cell.bottom() - 10.0),
                                    Stroke::new(1.0, color),
                                );
                            }
                        }
                        painter.hline(
                            header.x_range(),
                            header.bottom() - 0.5,
                            Stroke::new(1.0, r.divider_stroke_color_default),
                        );

                        // Body.
                        let row_h = cfg.row_height;
                        if page_rows.is_empty() {
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(row_w, row_h * 2.0), Sense::hover());
                            let visible_x = rect.x_range().intersection(ui.clip_rect().x_range());
                            ui.painter().text(
                                pos2(visible_x.center(), rect.center().y),
                                Align2::CENTER_CENTER,
                                labels.no_results,
                                theme.typography.body.font_id(),
                                r.text_fill_color_secondary,
                            );
                        } else {
                            ScrollArea::vertical()
                                .id_salt(id.with(("vscroll", state.page)))
                                .max_height(cfg.max_height)
                                .auto_shrink([false, true])
                                .show_rows(ui, row_h, page_rows.len(), |ui, range| {
                                    let viewport = ui.clip_rect();
                                    if let (Some(Drag::Row(_)), Some(p)) = (&drag, pointer) {
                                        // Scroll while dragging near the edges.
                                        if p.y < viewport.top() + 24.0 {
                                            ui.scroll_with_delta(vec2(0.0, 6.0));
                                        } else if p.y > viewport.bottom() - 24.0 {
                                            ui.scroll_with_delta(vec2(0.0, -6.0));
                                        }
                                    }
                                    for pos in range {
                                        let ri = page_rows[pos];
                                        let key = keys[ri];
                                        let row = &rows[ri];
                                        let (rect, _) = ui.allocate_exact_size(
                                            vec2(row_w, row_h),
                                            Sense::hover(),
                                        );
                                        row_rects.push((pos, rect));
                                        let resp = ui.interact(
                                            rect,
                                            id.with(("row", key)),
                                            Sense::click(),
                                        );
                                        let selected = state.selected.contains(&key);
                                        let dragged = drag == Some(Drag::Row(key));
                                        let accent =
                                            theme.accent_color.default_brush_for(theme.brightness);
                                        let fill = if dragged {
                                            r.subtle_fill_color_tertiary
                                        } else if selected {
                                            with_alpha(
                                                accent,
                                                if resp.hovered() { 0.16 } else { 0.11 },
                                            )
                                        } else if resp.hovered() {
                                            r.subtle_fill_color_secondary
                                        } else if cfg.striped && pos % 2 == 1 {
                                            r.control_alt_fill_color_secondary
                                        } else {
                                            r.subtle_fill_color_transparent
                                        };
                                        let fill = animate_color(
                                            ui.ctx(),
                                            resp.id.with("fill"),
                                            fill,
                                            theme.faster_animation_duration,
                                        );
                                        ui.painter().rect_filled(rect, 0.0, fill);
                                        if resp.clicked() {
                                            clicked_row = Some(ri);
                                            toggle_row(&mut state.selected, cfg.selection, key);
                                        }

                                        let mut x = rect.left();
                                        if cfg.row_reorder {
                                            let cell = Rect::from_min_size(
                                                pos2(x, rect.top()),
                                                vec2(HANDLE_WIDTH, row_h),
                                            );
                                            x += HANDLE_WIDTH;
                                            let handle = ui.interact(
                                                cell,
                                                id.with(("handle", key)),
                                                if reorder_rows {
                                                    Sense::drag()
                                                } else {
                                                    Sense::hover()
                                                },
                                            );
                                            let handle = handle.fluent_tooltip(if reorder_rows {
                                                labels.drag_to_reorder
                                            } else {
                                                labels.reorder_needs_unsorted
                                            });
                                            if reorder_rows && handle.drag_started() {
                                                drag = Some(Drag::Row(key));
                                            }
                                            if reorder_rows && (handle.hovered() || dragged) {
                                                ui.ctx().set_cursor_icon(if dragged {
                                                    CursorIcon::Grabbing
                                                } else {
                                                    CursorIcon::Grab
                                                });
                                            }
                                            let color = if !reorder_rows {
                                                r.text_fill_color_disabled
                                            } else if handle.hovered() || dragged {
                                                r.text_fill_color_primary
                                            } else {
                                                r.text_fill_color_tertiary
                                            };
                                            paint_grip(ui.painter(), cell.center(), color);
                                        }
                                        if multi {
                                            let cell = Rect::from_min_size(
                                                pos2(x, rect.top()),
                                                vec2(CHECK_WIDTH, row_h),
                                            );
                                            x += CHECK_WIDTH;
                                            if checkbox(
                                                ui,
                                                id.with(("check", key)),
                                                cell,
                                                Some(selected),
                                            )
                                            .clicked()
                                            {
                                                toggle_row(&mut state.selected, cfg.selection, key);
                                            }
                                        }
                                        for (vi, &ci) in visible.iter().enumerate() {
                                            let column = &cfg.columns[ci];
                                            let cell = Rect::from_min_size(
                                                pos2(x, rect.top()),
                                                vec2(widths[vi], row_h),
                                            );
                                            x += widths[vi];
                                            paint_cell(ui, &theme, column, row, cell, (key, ci));
                                        }
                                        if let Some(actions) = actions.as_mut() {
                                            let cell = Rect::from_min_size(
                                                pos2(rect.right() - ACTIONS_WIDTH, rect.top()),
                                                vec2(ACTIONS_WIDTH, row_h),
                                            );
                                            let mut child = ui.new_child(
                                                UiBuilder::new()
                                                    .id_salt(("actions", key))
                                                    .max_rect(cell)
                                                    .layout(Layout::left_to_right(Align::Center)),
                                            );
                                            child.add_space((ACTIONS_WIDTH - 28.0) / 2.0);
                                            let button = glyph_button(&mut child, 28.0, paint_more);
                                            let open = Popup::is_id_open(
                                                ui.ctx(),
                                                Popup::default_response_id(&button),
                                            );
                                            if !open {
                                                button.clone().fluent_tooltip(labels.open_menu);
                                            }
                                            let picked = Popup::menu(&button)
                                                .frame(flyout_frame(&theme))
                                                .gap(4.0)
                                                .width(180.0)
                                                .align(RectAlign::BOTTOM_END)
                                                .show(|ui| {
                                                    ui.spacing_mut().item_spacing.y = 2.0;
                                                    actions(ui, row)
                                                })
                                                .and_then(|r| r.inner);
                                            if let Some(picked) = picked {
                                                action = Some((ri, picked));
                                            }
                                        }
                                        if pos + 1 < page_rows.len() {
                                            ui.painter().hline(
                                                rect.x_range(),
                                                rect.bottom() - 0.5,
                                                Stroke::new(1.0, r.divider_stroke_color_default),
                                            );
                                        }
                                    }

                                    // Where a dragged row would land.
                                    if let (Some(Drag::Row(key)), Some(p)) = (&drag, pointer) {
                                        let slot = row_rects
                                            .iter()
                                            .find(|(_, rect)| p.y < rect.center().y)
                                            .map_or_else(
                                                || row_rects.last().map_or(0, |(pos, _)| pos + 1),
                                                |(pos, _)| *pos,
                                            );
                                        let y = row_rects
                                            .iter()
                                            .find(|(pos, _)| *pos == slot)
                                            .map_or_else(
                                                || {
                                                    row_rects
                                                        .last()
                                                        .map_or(0.0, |(_, r)| r.bottom())
                                                },
                                                |(_, r)| r.top(),
                                            );
                                        let x = row_rects
                                            .first()
                                            .map_or(0.0..=0.0, |(_, r)| r.x_range().into());
                                        paint_drop_line_h(ui.painter(), &theme, x, y);
                                        if released {
                                            let from = keys.iter().position(|k| k == key);
                                            let before = if slot < page_rows.len() {
                                                Some(page_rows[slot])
                                            } else {
                                                page_rows.last().map(|i| i + 1)
                                            };
                                            if let (Some(from), Some(before)) = (from, before) {
                                                rows_reordered =
                                                    from != before && from + 1 != before;
                                                move_item(rows, from, before);
                                            }
                                        }
                                    }
                                });
                        }

                        // Where a dragged column would land, and a ghost of
                        // its header under the pointer.
                        if let (Some(Drag::Column(dragged)), Some(p)) = (&drag, pointer) {
                            let slot = header_rects
                                .iter()
                                .filter(|(_, rect)| rect.center().x < p.x)
                                .count();
                            let x = header_rects
                                .get(slot)
                                .map(|(_, r)| r.left())
                                .or_else(|| header_rects.last().map(|(_, r)| r.right()));
                            let bottom = ui.min_rect().bottom();
                            if let (Some(x), Some(&(_, first))) = (x, header_rects.first()) {
                                paint_drop_line_v(ui.painter(), &theme, x, first.top()..=bottom);
                            }
                            if let Some(column) = cfg.columns.iter().find(|c| &c.id == dragged) {
                                paint_ghost(ui, &theme, &column.header, p);
                            }
                            if released {
                                let order = &mut state.order;
                                let from = order.iter().position(|c| c == dragged);
                                let before = match header_rects.get(slot) {
                                    Some(&(ci, _)) => {
                                        order.iter().position(|c| *c == cfg.columns[ci].id)
                                    }
                                    None => header_rects.last().and_then(|&(ci, _)| {
                                        order
                                            .iter()
                                            .position(|c| *c == cfg.columns[ci].id)
                                            .map(|i| i + 1)
                                    }),
                                };
                                if let (Some(from), Some(before)) = (from, before) {
                                    move_item(order, from, before);
                                }
                            }
                        }
                        table_rect = ui.min_rect();
                    });
            })
            .response;
        let _ = table_rect;

        if released {
            drag = None;
        } else if drag.is_some() {
            ui.ctx().request_repaint();
        }
        ui.data_mut(|d| match &drag {
            Some(drag) => {
                d.insert_temp(id.with("drag"), drag.clone());
            }
            None => d.remove::<Drag>(id.with("drag")),
        });

        // Footer: selection count and pagination.
        let selected_filtered = filtered
            .iter()
            .filter(|&&i| state.selected.contains(&keys[i]))
            .count();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let secondary = r.text_fill_color_secondary;
            if multi {
                ui.label(
                    theme
                        .typography
                        .body
                        .rich((labels.selected_of)(selected_filtered, filtered.len()))
                        .color(secondary),
                );
            }
            if !cfg.pagination {
                return;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let last = page_count - 1;
                let page = state.page;
                if page_button(ui, page < last, Paging::Last).clicked() {
                    state.page = last;
                }
                if page_button(ui, page < last, Paging::Next).clicked() {
                    state.page = page + 1;
                }
                if page_button(ui, page > 0, Paging::Previous).clicked() {
                    state.page = page - 1;
                }
                if page_button(ui, page > 0, Paging::First).clicked() {
                    state.page = 0;
                }
                ui.add_space(12.0);
                ui.label(
                    theme
                        .typography
                        .body
                        .rich((labels.page_of)(page + 1, page_count)),
                );
                ui.add_space(12.0);
                let mut size = state.page_size;
                ComboBox::new(id.with("page_size"), size.to_string())
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        for &s in &cfg.page_sizes {
                            combo_value(ui, &mut size, s, s.to_string());
                        }
                    });
                if size != state.page_size {
                    // Keep the first row of the current page in view.
                    state.page = state.page * state.page_size / size;
                    state.page_size = size;
                }
                ui.label(theme.typography.body.rich(labels.rows_per_page));
            });
        });

        let selection_changed = state.selected != selected_before;
        let selected = keys
            .iter()
            .enumerate()
            .filter(|(_, k)| state.selected.contains(k))
            .map(|(i, _)| i)
            .collect();
        let filtered_rows = filtered.len();
        state.store(ui.ctx(), id);
        DataTableResponse {
            response,
            id,
            action,
            selected,
            selection_changed,
            clicked_row,
            rows_reordered,
            filtered_rows,
        }
    }
}

fn toggle_row(selected: &mut HashSet<Id>, mode: DataTableSelectionMode, key: Id) {
    match mode {
        DataTableSelectionMode::None => {}
        DataTableSelectionMode::Single => {
            let was = selected.contains(&key);
            selected.clear();
            if !was {
                selected.insert(key);
            }
        }
        DataTableSelectionMode::Multiple => {
            if !selected.remove(&key) {
                selected.insert(key);
            }
        }
    }
}

/// A transparent button for toolbar actions such as "Reset".
fn subtle_button(theme: &FluentTheme) -> ButtonStyle {
    let mut style = ButtonStyle::icon(theme, super::IconButtonMode::Small);
    style.text_size = None;
    style
}

/// A 20px checkbox centered in `cell`.
fn checkbox(ui: &mut Ui, id: Id, cell: Rect, value: Option<bool>) -> Response {
    let boxed = Rect::from_center_size(cell.center(), Vec2::splat(20.0));
    let response = ui.interact(boxed.expand(4.0), id, Sense::click());
    if ui.is_rect_visible(boxed) {
        let state = ControlState::from_response(&response);
        paint_box(ui, id, boxed, state, value);
    }
    response
}

#[allow(clippy::too_many_arguments)]
fn paint_header<T>(
    ui: &Ui,
    painter: &Painter,
    theme: &FluentTheme,
    column: &Column<'_, T>,
    cell: Rect,
    sort: Option<(usize, SortDirection)>,
    numbered: bool,
    hovered: bool,
) {
    let r = &theme.resources;
    let indicator = if column.sortable { 18.0 } else { 0.0 };
    let max = (cell.width() - 2.0 * CELL_PADDING - indicator).max(0.0);
    let galley = WidgetText::from(column.header.as_str()).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        max,
        theme.typography.body_strong.font_id(),
    );
    let width = galley.size().x + indicator;
    let left = match column.align {
        Align::Min => cell.left() + CELL_PADDING,
        Align::Center => cell.center().x - width / 2.0,
        Align::Max => cell.right() - CELL_PADDING - width,
    };
    let color = if sort.is_some() {
        r.text_fill_color_primary
    } else {
        r.text_fill_color_secondary
    };
    let y = cell.center().y;
    painter
        .with_clip_rect(cell.intersect(painter.clip_rect()))
        .galley(pos2(left, y - galley.size().y / 2.0), galley.clone(), color);
    if !column.sortable {
        return;
    }
    let center = pos2(left + galley.size().x + 10.0, y);
    match sort {
        Some((index, dir)) => {
            match dir {
                SortDirection::Ascending => paint_chevron_up(painter, center, color),
                SortDirection::Descending => paint_chevron_down(painter, center, color),
            }
            if numbered {
                painter.text(
                    center + vec2(6.0, -4.0),
                    Align2::LEFT_CENTER,
                    (index + 1).to_string(),
                    egui::FontId::proportional(9.0),
                    r.text_fill_color_secondary,
                );
            }
        }
        None if hovered => paint_chevron_down(painter, center, r.text_fill_color_tertiary),
        None => {}
    }
}

fn paint_cell<T>(
    ui: &mut Ui,
    theme: &FluentTheme,
    column: &Column<'_, T>,
    row: &T,
    cell: Rect,
    salt: (Id, usize),
) {
    let clip = cell.intersect(ui.clip_rect());
    if !clip.is_positive() {
        return;
    }
    let inner = cell.shrink2(vec2(CELL_PADDING, 0.0));
    if let Some(cell_ui) = &column.cell_ui {
        let layout = match column.align {
            Align::Min => Layout::left_to_right(Align::Center),
            Align::Center => Layout::centered_and_justified(egui::Direction::LeftToRight),
            Align::Max => Layout::right_to_left(Align::Center),
        };
        let mut child = ui.new_child(
            UiBuilder::new()
                .id_salt(("cell", salt))
                .max_rect(inner)
                .layout(layout),
        );
        child.spacing_mut().item_spacing = vec2(6.0, 0.0);
        child.set_clip_rect(inner.intersect(clip));
        cell_ui(&mut child, row);
        return;
    }
    let galley = WidgetText::from((column.text)(row)).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        inner.width().max(0.0),
        theme.typography.body.font_id(),
    );
    let x = match column.align {
        Align::Min => inner.left(),
        Align::Center => inner.center().x - galley.size().x / 2.0,
        Align::Max => inner.right() - galley.size().x,
    };
    ui.painter().with_clip_rect(clip).galley(
        pos2(x, cell.center().y - galley.size().y / 2.0),
        galley,
        theme.resources.text_fill_color_primary,
    );
}

/// A drop-down button that opens a popup staying open while clicked in.
fn popup_button<R>(
    ui: &mut Ui,
    theme: &FluentTheme,
    text: String,
    width: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let mut style = ButtonStyle::standard(theme);
    style.padding.right += 22;
    let response = ui.add(Button::new(text).style(style));
    let state = ControlState::from_response(&response);
    let chevron = pos2(response.rect.right() - 18.0, response.rect.center().y);
    paint_chevron_down(ui.painter(), chevron, style.foreground.resolve(state));
    Popup::menu(&response)
        .frame(flyout_frame(theme))
        .gap(4.0)
        .width(width)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            add_contents(ui)
        })
        .map(|r| r.inner)
}

fn facet_filter<T>(
    ui: &mut Ui,
    theme: &FluentTheme,
    labels: &DataTableLabels,
    column: &Column<'_, T>,
    columns: &[Column<'_, T>],
    rows: &[T],
    state: &mut DataTableState,
) {
    let checked = state.facet_filters.get(&column.id).map_or(0, BTreeSet::len);
    let text = if checked > 0 {
        format!("{} · {}", column.header, (labels.selected_count)(checked))
    } else {
        column.header.clone()
    };
    popup_button(ui, theme, text, 220.0, |ui| {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for i in filter_rows(columns, rows, state, Some(&column.id)) {
            *counts.entry((column.text)(&rows[i])).or_default() += 1;
        }
        let options = column.facet_options.clone().unwrap_or_else(|| {
            let all: BTreeSet<String> = rows.iter().map(|row| (column.text)(row)).collect();
            all.into_iter().collect()
        });
        let values = state.facet_filters.entry(column.id.clone()).or_default();
        for option in options {
            let count = counts.get(&option).copied().unwrap_or(0);
            let on = values.contains(&option);
            if check_item(ui, &option, Some(count), on).clicked() && !values.remove(&option) {
                values.insert(option);
            }
        }
        if !values.is_empty() {
            menu_separator(ui);
            if ui.add(MenuItem::new(labels.clear_filters)).clicked() {
                values.clear();
                ui.close();
            }
        }
    });
}

fn column_picker<T>(
    ui: &mut Ui,
    theme: &FluentTheme,
    labels: &DataTableLabels,
    columns: &[Column<'_, T>],
    state: &mut DataTableState,
) {
    popup_button(ui, theme, labels.columns.to_owned(), 200.0, |ui| {
        for id in state.order.clone() {
            let Some(column) = columns.iter().find(|c| c.id == id) else {
                continue;
            };
            if !column.hideable {
                continue;
            }
            let shown = !state.hidden.contains(&id);
            if check_item(ui, &column.header, None, shown).clicked() && !state.hidden.remove(&id) {
                state.hidden.insert(id);
            }
        }
    });
}

/// A menu row with a checkbox, a label and an optional count on the right.
fn check_item(ui: &mut Ui, text: &str, count: Option<usize>, checked: bool) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let state = ControlState::from_response(&response);
        let fill = States::new(
            r.subtle_fill_color_transparent,
            r.subtle_fill_color_secondary,
            r.subtle_fill_color_tertiary,
            r.subtle_fill_color_transparent,
        )
        .resolve(state);
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, fill);
        let boxed =
            Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), Vec2::splat(18.0));
        paint_box(ui, response.id, boxed, state, Some(checked));
        let painter = ui.painter();
        painter.text(
            pos2(boxed.right() + 10.0, rect.center().y),
            Align2::LEFT_CENTER,
            text,
            theme.typography.body.font_id(),
            r.text_fill_color_primary,
        );
        if let Some(count) = count {
            painter.text(
                pos2(rect.right() - 12.0, rect.center().y),
                Align2::RIGHT_CENTER,
                count.to_string(),
                theme.typography.caption.font_id(),
                r.text_fill_color_secondary,
            );
        }
    }
    response
}

#[derive(Clone, Copy)]
enum Paging {
    First,
    Previous,
    Next,
    Last,
}

fn page_button(ui: &mut Ui, enabled: bool, paging: Paging) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let mut style = ButtonStyle::standard(&theme);
    style.padding = egui::Margin::same(0);
    let response = ui.add_enabled(
        enabled,
        Button::new("").style(style).min_size(Vec2::splat(32.0)),
    );
    let color = style
        .foreground
        .resolve(ControlState::from_response(&response));
    let c = response.rect.center();
    let painter = ui.painter();
    match paging {
        Paging::First => {
            paint_chevron_side(painter, c - vec2(2.5, 0.0), -1.0, color);
            paint_chevron_side(painter, c + vec2(2.5, 0.0), -1.0, color);
        }
        Paging::Previous => paint_chevron_side(painter, c, -1.0, color),
        Paging::Next => paint_chevron_side(painter, c, 1.0, color),
        Paging::Last => {
            paint_chevron_side(painter, c - vec2(2.5, 0.0), 1.0, color);
            paint_chevron_side(painter, c + vec2(2.5, 0.0), 1.0, color);
        }
    }
    response
}

/// A chevron pointing right (`dir` 1) or left (-1), 9px tall.
fn paint_chevron_side(painter: &Painter, center: Pos2, dir: f32, color: egui::Color32) {
    let points = vec![
        center + vec2(-2.25 * dir, -4.5),
        center + vec2(2.25 * dir, 0.0),
        center + vec2(-2.25 * dir, 4.5),
    ];
    painter.add(egui::epaint::PathShape::line(
        points,
        Stroke::new(1.25, color),
    ));
}

/// The `more` glyph: three dots.
fn paint_more(painter: &Painter, center: Pos2, color: egui::Color32) {
    for dx in [-5.0, 0.0, 5.0] {
        painter.circle_filled(center + vec2(dx, 0.0), 1.3, color);
    }
}

/// The `gripper_bar_vertical` glyph: two columns of three dots.
fn paint_grip(painter: &Painter, center: Pos2, color: egui::Color32) {
    for dx in [-2.5, 2.5] {
        for dy in [-5.0, 0.0, 5.0] {
            painter.circle_filled(center + vec2(dx, dy), 1.2, color);
        }
    }
}

fn paint_drop_line_h(
    painter: &Painter,
    theme: &FluentTheme,
    x: std::ops::RangeInclusive<f32>,
    y: f32,
) {
    let accent = theme.accent_color.default_brush_for(theme.brightness);
    painter.hline(x.clone(), y, Stroke::new(2.0, accent));
    painter.circle_stroke(pos2(*x.start() + 6.0, y), 3.0, Stroke::new(2.0, accent));
}

fn paint_drop_line_v(
    painter: &Painter,
    theme: &FluentTheme,
    x: f32,
    y: std::ops::RangeInclusive<f32>,
) {
    let accent = theme.accent_color.default_brush_for(theme.brightness);
    painter.vline(x, y, Stroke::new(2.0, accent));
}

/// A floating copy of a dragged header under the pointer.
fn paint_ghost(ui: &Ui, theme: &FluentTheme, text: &str, at: Pos2) {
    let painter = ui.ctx().layer_painter(LayerId::new(
        Order::Tooltip,
        Id::unique("fluent_egui::DataTable::ghost"),
    ));
    let galley = painter.layout_no_wrap(
        text.to_owned(),
        theme.typography.body_strong.font_id(),
        theme.resources.text_fill_color_primary,
    );
    let rect = Rect::from_min_size(at + vec2(12.0, 8.0), galley.size() + vec2(24.0, 14.0));
    painter.add(flyout_frame(theme).inner_margin(0).paint(rect));
    painter.galley(
        rect.min + vec2(12.0, 7.0),
        galley,
        theme.resources.text_fill_color_primary,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Row {
        name: &'static str,
        kind: &'static str,
        n: u32,
    }

    fn data() -> Vec<Row> {
        vec![
            Row {
                name: "beta",
                kind: "x",
                n: 10,
            },
            Row {
                name: "Alpha",
                kind: "y",
                n: 2,
            },
            Row {
                name: "gamma",
                kind: "x",
                n: 2,
            },
            Row {
                name: "delta",
                kind: "z",
                n: 33,
            },
        ]
    }

    fn columns<'a>() -> Vec<Column<'a, Row>> {
        vec![
            Column::new("name", "Name", |r: &Row| r.name.to_owned()).text_filter(),
            Column::new("kind", "Kind", |r: &Row| r.kind.to_owned()).faceted(),
            Column::new("n", "N", |r: &Row| r.n.to_string()),
        ]
    }

    #[test]
    fn sorts_numbers_as_numbers_and_text_ignoring_case() {
        let (rows, cols) = (data(), columns());
        let mut idx: Vec<usize> = (0..rows.len()).collect();
        sort_rows(
            &cols,
            &rows,
            &mut idx,
            &[("n".into(), SortDirection::Ascending)],
        );
        assert_eq!(idx, [1, 2, 0, 3]);
        sort_rows(
            &cols,
            &rows,
            &mut idx,
            &[("name".into(), SortDirection::Descending)],
        );
        assert_eq!(idx, [2, 3, 0, 1]);
    }

    #[test]
    fn multi_sort_breaks_ties_with_later_columns() {
        let (rows, cols) = (data(), columns());
        let mut idx: Vec<usize> = (0..rows.len()).collect();
        let sorting = [
            ("n".into(), SortDirection::Ascending),
            ("name".into(), SortDirection::Descending),
        ];
        sort_rows(&cols, &rows, &mut idx, &sorting);
        assert_eq!(idx, [2, 1, 0, 3]);
    }

    #[test]
    fn filters_combine() {
        let (rows, cols) = (data(), columns());
        let mut state = DataTableState {
            search: "TA".into(),
            ..Default::default()
        };
        assert_eq!(filter_rows(&cols, &rows, &state, None), [0, 3]);
        state.search.clear();
        state
            .facet_filters
            .insert("kind".into(), ["x".to_owned()].into());
        assert_eq!(filter_rows(&cols, &rows, &state, None), [0, 2]);
        state.text_filters.insert("name".into(), "GAM".into());
        assert_eq!(filter_rows(&cols, &rows, &state, None), [2]);
        // A facet's own filter is left out when counting its values.
        assert_eq!(filter_rows(&cols, &rows, &state, Some("kind")), [2]);
        state.text_filters.clear();
        assert_eq!(
            filter_rows(&cols, &rows, &state, Some("kind")),
            [0, 1, 2, 3]
        );
    }

    #[test]
    fn hidden_columns_are_not_searched() {
        let (rows, cols) = (data(), columns());
        let mut state = DataTableState {
            search: "33".into(),
            ..Default::default()
        };
        assert_eq!(filter_rows(&cols, &rows, &state, None), [3]);
        state.hidden.insert("n".into());
        assert!(filter_rows(&cols, &rows, &state, None).is_empty());
    }

    #[test]
    fn toggle_sort_cycles_and_multi_sort_appends() {
        let mut state = DataTableState::default();
        state.toggle_sort("a", false);
        assert_eq!(state.sorting, [("a".into(), SortDirection::Ascending)]);
        state.toggle_sort("b", true);
        state.toggle_sort("a", true);
        assert_eq!(
            state.sorting,
            [
                ("a".into(), SortDirection::Descending),
                ("b".into(), SortDirection::Ascending)
            ]
        );
        state.toggle_sort("a", true);
        assert_eq!(state.sorting, [("b".into(), SortDirection::Ascending)]);
        state.toggle_sort("c", false);
        state.toggle_sort("c", false);
        state.toggle_sort("c", false);
        assert!(state.sorting.is_empty());
    }

    #[test]
    fn move_item_lands_before_the_target() {
        let mut v = vec![0, 1, 2, 3, 4];
        move_item(&mut v, 0, 3);
        assert_eq!(v, [1, 2, 0, 3, 4]);
        move_item(&mut v, 4, 0);
        assert_eq!(v, [4, 1, 2, 0, 3]);
        move_item(&mut v, 1, 5);
        assert_eq!(v, [4, 2, 0, 3, 1]);
        move_item(&mut v, 2, 3);
        assert_eq!(v, [4, 2, 0, 3, 1]);
    }
}

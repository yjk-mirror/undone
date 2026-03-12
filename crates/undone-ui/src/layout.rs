pub const DEFAULT_WINDOW_WIDTH: f64 = 1200.0;
pub const DEFAULT_WINDOW_HEIGHT: f64 = 800.0;
pub const CUSTOM_TITLE_BAR_HEIGHT: f64 = 40.0;
pub const STORY_COLUMN_MAX_WIDTH: f64 = 680.0;
pub const ACTION_BUTTON_MIN_WIDTH: f64 = 240.0;

const ACTION_BUTTON_OUTER_WIDTH: f64 = ACTION_BUTTON_MIN_WIDTH + 8.0;
const ACTION_BAR_CHROME_HEIGHT: f64 = 25.0;
const ACTION_ROW_HEIGHT: f64 = 56.0;
const DETAIL_STRIP_HEIGHT: f64 = 41.0;
const STORY_MIN_HEIGHT: f64 = 200.0;

pub fn sidebar_width_for_window(window_width: f64) -> f64 {
    if window_width < 720.0 {
        180.0
    } else if window_width < 1024.0 {
        220.0
    } else {
        280.0
    }
}

pub fn story_region_width_for_window(window_width: f64) -> f64 {
    (window_width - sidebar_width_for_window(window_width)).max(ACTION_BUTTON_MIN_WIDTH)
}

pub fn action_button_columns_for_window(window_width: f64) -> usize {
    let usable_width = story_region_width_for_window(window_width);
    let columns = ((usable_width + 8.0) / ACTION_BUTTON_OUTER_WIDTH).floor() as usize;
    columns.max(1)
}

pub fn action_button_rows_for_window(window_width: f64, action_count: usize) -> usize {
    let count = action_count.max(1);
    let columns = action_button_columns_for_window(window_width);
    count.div_ceil(columns)
}

pub fn action_bar_height_for_window(window_width: f64, action_count: usize) -> f64 {
    ACTION_BAR_CHROME_HEIGHT
        + ACTION_ROW_HEIGHT * action_button_rows_for_window(window_width, action_count) as f64
}

pub fn story_panel_max_height(window_width: f64, window_height: f64, action_count: usize) -> f64 {
    let available_height = (window_height - CUSTOM_TITLE_BAR_HEIGHT).max(STORY_MIN_HEIGHT);
    (available_height
        - DETAIL_STRIP_HEIGHT
        - action_bar_height_for_window(window_width, action_count))
    .max(STORY_MIN_HEIGHT)
}

use crate::config::KeyConfig;

static CMD_GROUP_GENERAL: &str = "-- General --";
#[allow(dead_code)]
static CMD_GROUP_TABLE: &str = "-- Table --";
static CMD_GROUP_DATABASES: &str = "-- Databases --";
#[allow(dead_code)]
static CMD_GROUP_PROPERTIES: &str = "-- Properties --";

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct CommandText {
    pub name: String,
    pub group: &'static str,
    pub hide_help: bool,
}

impl CommandText {
    pub const fn new(name: String, group: &'static str) -> Self {
        Self {
            name,
            group,
            hide_help: false,
        }
    }
}

pub struct CommandInfo {
    pub text: CommandText,
}

impl CommandInfo {
    pub const fn new(text: CommandText) -> Self {
        Self { text }
    }
}


pub fn scroll_up_down_multiple_lines(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll up/down multiple lines [{},{}]",
            key.scroll_up_multiple_lines, key.scroll_down_multiple_lines,
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn scroll_to_top_bottom(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll to top/bottom [{},{}]",
            key.scroll_to_top, key.scroll_to_bottom,
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn expand_collapse(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Expand/Collapse [{},{}]", key.scroll_right, key.scroll_left,),
        CMD_GROUP_DATABASES,
    )
}

pub fn filter(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("Filter [{}]", key.filter), CMD_GROUP_GENERAL)
}

pub fn move_focus(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Move focus to left/right [{},{}]",
            key.focus_left, key.focus_right
        ),
        CMD_GROUP_GENERAL,
    )
}


pub fn extend_or_shorten_widget_width(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Extend/shorten widget width to left/right [{},{}]",
            key.extend_or_shorten_widget_width_to_left, key.extend_or_shorten_widget_width_to_right
        ),
        CMD_GROUP_GENERAL,
    )
}


pub fn toggle_tabs(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Tab [{},{},{}]",
            key_config.tab_records, key_config.tab_properties, key_config.tab_sql_editor
        ),
        CMD_GROUP_GENERAL,
    )
}


pub fn help(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Help [{}]", key_config.open_help),
        CMD_GROUP_GENERAL,
    )
}

pub fn exit_pop_up(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Exit pop up [{}]", key_config.exit_popup),
        CMD_GROUP_GENERAL,
    )
}

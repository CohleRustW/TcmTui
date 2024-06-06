use ratatui::{
    prelude::*,
    style::Modifier,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{config::KeyConfig, event::Key};

use super::{
    command::{self, CommandInfo},
    tabs::SelectedTab,
    Component, DrawableComponent, EventState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Tcm,
    Text,
}
pub struct QueryInputComponents {
    pub query_type: QueryType,
    pub input: String,
    /// Current value of the input box
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
    /// History of recorded messages
    pub history: Vec<String>,
    pub key_config: KeyConfig,
    pub tab: SelectedTab,
    pub color: Color,
}

impl QueryInputComponents {
    pub const fn new(key_config: KeyConfig) -> Self {
        Self {
            query_type: QueryType::Tcm,
            input: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            key_config: key_config,
            tab: SelectedTab::Tab1,
            color: Color::Reset,
        }
    }
    fn swich_query_type(&mut self) {
        match self.query_type {
            QueryType::Tcm => self.query_type = QueryType::Text,
            QueryType::Text => self.query_type = QueryType::Tcm,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn submit_message(&mut self) {
        self.history.insert(0, self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }
}

impl DrawableComponent for QueryInputComponents {
    fn draw(&self, f: &mut Frame, _area: Rect, _focused: bool) -> anyhow::Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(20),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(_area);
        let history_area = chunks[0];
        let query_type_area = chunks[1];
        let input_area = chunks[2];

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default())
            .add_modifier(Modifier::REVERSED)
            .fg(self.color)
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, input_area);

        // Make the cursor visible and ask ratatui to put it at the specified coordinates after
        // rendering
        #[allow(clippy::cast_possible_truncation)]
        f.set_cursor(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            input_area.x + self.cursor_position as u16 + 1,
            // Move one line down, from the border to the input line
            input_area.y + 1,
        );

        let messages: Vec<ListItem> = self
            .history
            .iter()
            .enumerate()
            .map(|(_, m)| {
                let content = Line::from(Span::raw(format!("{m}")));
                ListItem::new(content)
            })
            .collect();
        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Commit"));
        f.render_widget(messages, history_area);
        let button_states = [QueryType::Tcm, QueryType::Text];
        draw_query_type_ui(f, button_states, query_type_area, &self);
        Ok(())
    }
}

impl Component for QueryInputComponents {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::expand_collapse(&self.key_config)))
    }
    fn event(&mut self, key: crate::event::Key) -> anyhow::Result<EventState> {
        match key {
            Key::Enter => self.submit_message(),
            Key::Up => self.swich_query_type(),
            Key::Down => self.swich_query_type(),
            Key::Char(to_insert) => {
                self.enter_char(to_insert);
            }
            Key::Backspace => {
                self.delete_char();
            }
            Key::Left => {
                self.move_cursor_left();
            }
            Key::Right => {
                self.move_cursor_right();
            }
            _ => return Ok(EventState::NotConsumed),
        }
        Ok(EventState::Consumed)
    }
}

fn draw_query_type_ui(
    frame: &mut Frame,
    states: [QueryType; 2],
    area: Rect,
    component: &QueryInputComponents,
) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Max(3),
                Constraint::Length(1),
                // Constraint::Min(0), // ignore remaining space
            ]
            .as_ref(),
        )
        .split(area);
    let [title, buttons, help] = [vertical[0], vertical[1], vertical[2]];

    frame.render_widget(Paragraph::new("查询方式切换"), title);
    render_buttons(frame, buttons, states, component);
    frame.render_widget(Paragraph::new("⬇️/⬆️: Select search mode"), help);
}

fn render_buttons(
    frame: &mut Frame<'_>,
    area: Rect,
    states: [QueryType; 2],
    component: &QueryInputComponents,
) {
    let horizontal = Layout::horizontal([
        Constraint::Length(15),
        // Constraint::Min(0), // ignore remaining space
    ]);
    let [green] = horizontal.areas(area);
    let button_str: &str;
    match component.query_type {
        QueryType::Tcm => button_str = "TCM查询模式",
        QueryType::Text => button_str = "关键词搜索",
    }

    frame.render_widget(Button::new(button_str).theme(GREEN).state(states[0]), green);
}

#[derive(Debug, Clone)]
struct Button<'a> {
    label: Line<'a>,
    theme: Theme,
    query_type: QueryType,
}

#[derive(Debug, Clone, Copy)]
struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
}

const BLUE: Theme = Theme {
    text: Color::Rgb(16, 24, 48),
    background: Color::Rgb(48, 72, 144),
    highlight: Color::Rgb(64, 96, 192),
    shadow: Color::Rgb(32, 48, 96),
};

const GREEN: Theme = Theme {
    text: Color::Rgb(16, 48, 16),
    background: Color::Rgb(48, 144, 48),
    highlight: Color::Rgb(64, 192, 64),
    shadow: Color::Rgb(32, 96, 32),
};
impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Button {
            label: label.into(),
            theme: BLUE,
            query_type: QueryType::Tcm,
        }
    }

    pub const fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub const fn state(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }
}

impl<'a> Widget for Button<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (background, text, shadow, highlight) = self.colors();
        buf.set_style(area, Style::new().bg(background).fg(text));

        // render top line if there's enough space
        if area.height > 2 {
            buf.set_string(
                area.x,
                area.y,
                "▔".repeat(area.width as usize),
                Style::new().fg(highlight).bg(background),
            );
        }
        // render bottom line if there's enough space
        if area.height > 1 {
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                "▁".repeat(area.width as usize),
                Style::new().fg(shadow).bg(background),
            );
        }
        // render label centered
        buf.set_line(
            area.x + (area.width.saturating_sub(self.label.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &self.label,
            area.width,
        );
    }
}

impl Button<'_> {
    const fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.query_type {
            QueryType::Tcm => (theme.background, theme.text, theme.shadow, theme.highlight),
            QueryType::Text => (theme.highlight, theme.text, theme.shadow, theme.highlight),
        }
    }
}

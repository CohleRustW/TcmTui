use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub struct HelpComponent {
    cmds: Vec<CommandInfo>,
    visible: bool,
    selection: u16,
    key_config: KeyConfig,
}

impl DrawableComponent for HelpComponent {
    fn draw(&self, f: &mut Frame, _area: Rect, _focused: bool) -> Result<()> {
        if self.visible {
            const SIZE: (u16, u16) = (65, 24);
            let scroll_threshold = SIZE.1 / 3;
            let scroll = self.selection.saturating_sub(scroll_threshold);

            let area = Rect::new(
                (f.size().width.saturating_sub(SIZE.0)) / 2,
                (f.size().height.saturating_sub(SIZE.1)) / 2,
                SIZE.0.min(f.size().width),
                SIZE.1.min(f.size().height),
            );

            f.render_widget(Clear, area);
            f.render_widget(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
                area,
            );

            let chunks = Layout::default()
                .vertical_margin(1)
                .horizontal_margin(1)
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
                .split(area);

            f.render_widget(
                Paragraph::new(Span::styled(format!("tcm baby walker {}", "0.1"), Style::default())).scroll((scroll, 0)),
                chunks[0],
            );

            f.render_widget(
                Paragraph::new(Span::styled(format!("tcm baby walker {}", "0.1"), Style::default()))
                    .alignment(Alignment::Right),
                chunks[1],
            );
        }

        Ok(())
    }
}

impl Component for HelpComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        if self.visible {
            if key == self.key_config.exit_popup {
                self.hide();
                return Ok(EventState::Consumed);
            } else if key == self.key_config.scroll_down {
                self.scroll_selection(true);
                return Ok(EventState::Consumed);
            } else if key == self.key_config.scroll_up {
                self.scroll_selection(false);
                return Ok(EventState::Consumed);
            }
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.open_help {
            self.show()?;
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    fn show(&mut self) -> Result<()> {
        self.visible = true;

        Ok(())
    }
}

impl HelpComponent {
    pub const fn new(key_config: KeyConfig) -> Self {
        Self {
            cmds: vec![],
            visible: false,
            selection: 0,
            key_config,
        }
    }

    pub fn set_cmds(&mut self, cmds: Vec<CommandInfo>) {
        self.cmds = cmds
            .into_iter()
            .filter(|e| !e.text.hide_help)
            .collect::<Vec<_>>();
    }

    fn scroll_selection(&mut self, inc: bool) {
        let mut new_selection = self.selection;

        new_selection = if inc {
            new_selection.saturating_add(1)
        } else {
            new_selection.saturating_sub(1)
        };
        new_selection = new_selection.max(0);

        self.selection = new_selection.min(self.cmds.len().saturating_sub(1) as u16);
    }
}

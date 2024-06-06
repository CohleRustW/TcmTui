pub mod error;
pub mod help;
pub mod tabs;
pub mod proc;
pub mod command;
pub mod syntax_text;
pub mod host;
pub mod footer;
use anyhow::Result;
pub mod query_input;
pub mod total_proc;
use async_trait::async_trait;
use ratatui::{backend::Backend, layout::Rect, Frame};

use self::command::CommandInfo;

#[derive(PartialEq, Debug)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl EventState {
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
    }
}

impl From<bool> for EventState {
    fn from(consumed: bool) -> Self {
        if consumed {
            Self::Consumed
        } else {
            Self::NotConsumed
        }
    }
}

pub trait DrawableComponent {
    fn draw(&self, f: &mut Frame, rect: Rect, focused: bool) -> Result<()>;
}

pub trait StatefulDrawableComponent {
    fn draw(&self, f: &mut Frame, rect: Rect, focused: bool) -> Result<()>;
}

pub trait MovableComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame,
        rect: Rect,
        focused: bool,
        x: u16,
        y: u16,
    ) -> Result<()>;
}

/// base component trait
#[async_trait]
pub trait Component {
    fn commands(&self, out: &mut Vec<CommandInfo>);

    fn event(&mut self, key: crate::event::Key) -> Result<EventState>;

    async fn async_event(
        &mut self,
        _key: crate::event::Key,
        // _pool: &SqlitePool,
    ) -> Result<EventState> {
        Ok(EventState::NotConsumed)
    }

    fn focused(&self) -> bool {
        false
    }

    fn focus(&mut self, _focus: bool) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn hide(&mut self) {}

    fn show(&mut self) -> Result<()> {
        Ok(())
    }

    fn toggle_visible(&mut self) -> Result<()> {
        if self.is_visible() {
            self.hide();
            Ok(())
        } else {
            self.show()
        }
    }
}

#![allow(clippy::wildcard_imports, clippy::enum_glob_use)]

use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::config::KeyConfig;

use super::{command::{self, CommandInfo}, Component, DrawableComponent, EventState};

#[derive(Default)]
pub struct TabComponent {
    state: TabState,
    pub selected_tab: SelectedTab,
    key_config: KeyConfig,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum TabState {
    #[default]
    Running,
    Quitting,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = "主机搜索 [1]")]
    Tab1,
    #[strum(to_string = "进程搜索 [2]")]
    Tab2,
}

impl DrawableComponent for TabComponent {
    fn draw(&self, f: &mut Frame, _area: Rect, _focused: bool) -> anyhow::Result<()> {
        f.render_widget(self, _area);
        Ok(())
    }
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

impl Widget for &TabComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;
        let vertical = Layout::vertical([Length(1), Length(5)]);
        let [header_area, inner_area ] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Min(5)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);
        self.selected_tab.render(inner_area, buf);
        // render_footer(footer_area, buf);
    }
}

impl TabComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            state: TabState::Running,
            selected_tab: SelectedTab::Tab1,
            key_config,
        }
    }
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = (Color::default(), self.selected_tab.palette().c700);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }
    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    pub fn quit(&mut self) {
        self.state = TabState::Quitting;
    }
}

fn render_title(area: Rect, buf: &mut Buffer) {
    "Tcm Baby Walker".bold().render(area, buf);
}


impl Widget for SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // in a real app these might be separate widgets
        match self {
            Self::Tab1 => self.render_tab0(area, buf),
            Self::Tab2 => self.render_tab1(area, buf),
        }
    }
}

impl SelectedTab {
    /// Return tab's name as a styled `Line`
    fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    fn render_tab0(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("检索主机信息, Tab 键切换选中主机模式")
            .block(self.block())
            .render(area, buf);
    }

    fn render_tab1(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("更详细的 Tcm 进程信息检索, 支持文件查看, Tab 键切换到检索内容选中模式")
            .block(self.block())
            .render(area, buf);
    }


    /// A block surrounding the tab's content
    fn block(self) -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette().c700)
    }

    const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Tab1 => tailwind::BLUE,
            Self::Tab2 => tailwind::EMERALD,
        }
    }
}


impl Component for TabComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::expand_collapse(&self.key_config)))
    }
    fn event(&mut self, _key: crate::event::Key) -> anyhow::Result<EventState> {
        Ok(EventState::NotConsumed)
    }
}

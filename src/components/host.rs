use crate::components::command::{self, CommandInfo};
use crate::config::KeyConfig;
use crate::database::select_all_host;
use crate::event::Key;
use crate::tools::host::HostInfo;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{ScrollbarState, TableState};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    Frame,
};
use ratatui::{prelude::*, widgets::*};
use sqlx::SqlitePool;
use unicode_width::UnicodeWidthStr;

use super::{Component, DrawableComponent, EventState};

// ▸

pub const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

const ITEM_HEIGHT: usize = 4;

impl HostInfo {
    fn ref_array(&self) -> [&str; 4] {
        [
            &self.inner_ip,
            &self.world_id,
            &self.zone_id,
            &self.host_name,
        ]
    }

    fn inner_ip(&self) -> &str {
        &self.inner_ip
    }

    fn world_id(&self) -> &str {
        &self.world_id
    }

    fn zone_id(&self) -> &str {
        &self.zone_id
    }
    fn host_name(&self) -> &str {
        &self.host_name
    }
}

#[derive(Clone)]
pub struct TableColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub footer_border_color: Color,
}

impl TableColors {
    pub const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

#[derive(PartialEq)]
pub enum Focus {
    #[allow(dead_code)]
    Filter,
    #[allow(dead_code)]
    Tree,
}

#[derive(Clone)]
pub struct HostInfoComponent {
    state: TableState,
    pub items: Vec<HostInfo>,
    longest_item_lens: (u16, u16, u16, u16), // order is (name, address, email)
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    key_config: KeyConfig,
}

impl DrawableComponent for HostInfoComponent {
    fn draw(&self, f: &mut Frame, _area: Rect, _focused: bool) -> anyhow::Result<()> {
        // let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());
        let mut s = self.clone();

        s.set_colors();

        s.render_table(f, _area);

        s.render_scrollbar(f, _area);

        // s.render_footer(f, rects[1]);
        Ok(())
    }
}

impl HostInfoComponent {
    pub async fn new(db: &SqlitePool, key_config: KeyConfig) -> Result<Self, sqlx::Error> {
        let data_vec = select_all_host(&db).await?;
        Ok(Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(&data_vec),
            scroll_state: ScrollbarState::new((data_vec.len() - 1) * ITEM_HEIGHT),
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            items: data_vec,
            key_config: key_config,
        })
    }
    fn render_table(&mut self, f: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_style_fg);

        let header = ["InnerIp", "WorldID", "ZoneID", "InstID", "HostName"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => self.colors.normal_row_color,
                _ => self.colors.alt_row_color,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Min(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1 + 1),
                Constraint::Min(self.longest_item_lens.2),
                Constraint::Min(self.longest_item_lens.3),
            ],
        )
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(Text::from(vec![
            "".into(),
            bar.into(),
            bar.into(),
            "".into(),
        ]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
        f.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, f: &mut Frame, area: Rect) {
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }
}

fn constraint_len_calculator(items: &[HostInfo]) -> (u16, u16, u16, u16) {
    let inner_ip_len = items
        .iter()
        .map(HostInfo::inner_ip)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let world_id_len = items
        .iter()
        .map(HostInfo::world_id)
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let zone_id_len = items
        .iter()
        .map(HostInfo::zone_id)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let host_name_len = items
        .iter()
        .map(HostInfo::host_name)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (
        inner_ip_len as u16,
        world_id_len as u16,
        zone_id_len as u16,
        host_name_len as u16,
    )
}
impl HostInfoComponent {
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }
    // pub fn search_host_block(&self) -> Result<Vec<HostInfo>, Box<dyn std::error::Error>> {
    //     use tokio::runtime::Runtime;
    //     let runtime = Runtime::new().unwrap();
    //     let hosts = runtime.block_on(self.search_host());
    //     hosts
    // }
}

impl Component for HostInfoComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::expand_collapse(&self.key_config)))
    }
    fn event(&mut self, key: crate::event::Key) -> anyhow::Result<EventState> {
        // use KeyCode::*;
        match key {
            // Key::Char('q') | Key::Esc => return Ok(EventState::NotConsumed),
            Key::Char('j') | Key::Down => self.next(),
            Key::Char('k') | Key::Up => self.previous(),
            Key::Char('l') | Key::Right => self.next_color(),
            Key::Char('h') | Key::Left => self.previous_color(),
            _ => return Ok(EventState::NotConsumed),
        };
        Ok(EventState::Consumed)
    }
}

use crate::components::command::{self, CommandInfo};
use crate::config::KeyConfig;
use crate::database::query_hosts_sql;
use crate::event::Key;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{ScrollbarState, TableState};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    Frame,
};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use unicode_width::UnicodeWidthStr;

use super::{Component, DrawableComponent, EventState};
use crate::database::TcmQueryResult;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TotalProc {
    pub func_id: String,
    pub inst_id: String,
    pub proc_name: String,
    pub group_name: String,
    pub inner_ip: String,
    pub host_name: String,
    pub world_id: String,
    pub zone_id: String,
    pub work_path: String,
    pub func_name: String,
}

impl From<&TcmQueryResult> for TotalProc {
    fn from(value: &TcmQueryResult) -> Self {
        TotalProc {
            func_id: value.func_id.to_string(),
            proc_name: value.proc_name.clone(),
            group_name: value.proc_group_name.clone(),
            inner_ip: value.inner_ip.clone(),
            host_name: value.host_name.clone(),
            world_id: value.world_id.clone(),
            zone_id: value.zone_id.clone(),
            work_path: value.work_path.clone(),
            func_name: value.func_name.clone(),
            inst_id: value.inst_id.to_string(),
        }
    }
}

pub const PROC_PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

const PROC_ITEM_HEIGHT: usize = 4;
impl TotalProc {
    fn ref_array(&self) -> [&str; 10] {
        [
            &self.func_id,
            &self.inst_id,
            &self.proc_name,
            &self.group_name,
            &self.inner_ip,
            &self.host_name,
            &self.world_id,
            &self.zone_id,
            &self.work_path,
            &self.func_name,
        ]
    }

    fn work_path(&self) -> &str {
        &self.work_path
    }

    fn proc_name(&self) -> &str {
        &self.proc_name
    }

    fn func_id(&self) -> &str {
        &self.func_id
    }
    fn host_name(&self) -> &str {
        &self.host_name
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

    fn inst_id(&self) -> &str {
        &self.inst_id
    }
    fn func_name(&self) -> &str {
        &self.func_name
    }
    fn group_name(&self) -> &str {
        &self.group_name
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
pub struct TotalProcInfoComponent {
    state: TableState,
    pub select_item: Option<TotalProc>,
    pub items: Vec<TotalProc>,
    longest_item_lens: (u16, u16, u16, u16, u16, u16, u16, u16, u16, u16), // order is (name, address, email)
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    key_config: KeyConfig,
}

impl DrawableComponent for TotalProcInfoComponent {
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

impl TotalProcInfoComponent {
    pub async fn new(db: &SqlitePool, key_config: KeyConfig) -> anyhow::Result<Self> {
        let c = query_hosts_sql("*.*.*.*", db).await?;
        let data_vec = c.iter().map(|f| f.into()).collect::<Vec<TotalProc>>();
        Ok(Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(&data_vec),
            scroll_state: ScrollbarState::new((data_vec.len() - 1) * PROC_ITEM_HEIGHT),
            colors: TableColors::new(&PROC_PALETTES[0]),
            color_index: 0,
            items: data_vec,
            key_config: key_config.clone(),
            select_item: None
        })
    }
    fn render_table(&mut self, f: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_style_fg);

        let header = [
            "funcID",
            "InstID",
            "执行文件名",
            "进程组",
            "InnerIP",
            "HostName",
            "WorldID",
            "ZoneID",
            "WorkPath",
            "FuncName",
        ]
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
                .map(|content| Cell::from(Text::from(format!("\n{content:<20}\n")))) // Increase width to 20
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Min(self.longest_item_lens.0),
                Constraint::Min(self.longest_item_lens.1),
                Constraint::Min(self.longest_item_lens.2),
                Constraint::Min(self.longest_item_lens.3),
                Constraint::Min(self.longest_item_lens.4),
                Constraint::Min(self.longest_item_lens.5),
                Constraint::Min(self.longest_item_lens.6),
                Constraint::Min(self.longest_item_lens.7),
                Constraint::Min(self.longest_item_lens.8),
                Constraint::Min(self.longest_item_lens.9),
                // Constraint::Min(self.longest_item_lens.4),
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

fn constraint_len_calculator(
    items: &[TotalProc],
) -> (u16, u16, u16, u16, u16, u16, u16, u16, u16, u16) {
    let proc_name_len = items
        .iter()
        .map(TotalProc::proc_name)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let _world_id_len = items
        .iter()
        .map(TotalProc::world_id)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let _zone_id_len = items
        .iter()
        .map(TotalProc::zone_id)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let _inst_id_len = items
        .iter()
        .map(TotalProc::inst_id)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let proc_group_name = items
        .iter()
        .map(TotalProc::group_name)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let work_path_len = items
        .iter()
        .map(TotalProc::work_path)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let func_name = items
        .iter()
        .map(TotalProc::func_name)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let inner_ip_len = items
        .iter()
        .map(TotalProc::inner_ip)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let host_name_len = items
        .iter()
        .map(TotalProc::host_name)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let _fun_id_len = items
        .iter()
        .map(TotalProc::func_id)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (
        3 as u16,
        inner_ip_len as u16,
        host_name_len as u16,
        // world_id_len as u16,
        3 as u16,
        // zone_id_len as u16,
        3 as u16,
        // inst_id_len as u16,
        3 as u16,
        proc_name_len as u16,
        func_name as u16,
        work_path_len as u16,
        proc_group_name as u16,
    )
}
impl TotalProcInfoComponent {
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
        self.scroll_state = self.scroll_state.position(i * PROC_ITEM_HEIGHT);
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PROC_PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PROC_PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PROC_PALETTES[self.color_index]);
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
        self.scroll_state = self.scroll_state.position(i * PROC_ITEM_HEIGHT);
        self.select_item = Some(self.items[i].clone());
    }
}

impl Component for TotalProcInfoComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::expand_collapse(&self.key_config)))
    }
    fn event(&mut self, key: crate::event::Key) -> anyhow::Result<EventState> {
        // use KeyCode::*;
        match key {
            Key::Char('j') | Key::Down => self.next(),
            Key::Char('k') | Key::Up => self.previous(),
            Key::Char('l') | Key::Right => self.next_color(),
            Key::Char('h') | Key::Left => self.previous_color(),
            _ => return Ok(EventState::NotConsumed),
        };
        Ok(EventState::Consumed)
    }
}

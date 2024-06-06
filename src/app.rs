use super::components::Component;
use crate::components::footer::FooterComponent;
use crate::components::host::{HostInfoComponent, TableColors, PALETTES};
use crate::components::proc::ProcInfoComponent;
use crate::components::query_input::{QueryInputComponents, QueryType};
use crate::components::syntax_text::SyntaxTextComponent;
use crate::components::tabs::{SelectedTab, TabComponent};
use crate::components::total_proc::{TotalProc, TotalProcInfoComponent};
use crate::components::{DrawableComponent, EventState};
use crate::config;
use crate::database::{query_hosts_sql, select_all_host};
use crate::event::Event;
use crate::tools::host::HostInfo;
use crate::tools::search::search_vec;
use crate::utils::drop_app;
use crate::{
    components::{
        command::{self, CommandInfo},
        error::ErrorComponent,
        help::HelpComponent,
    },
    config::KeyConfig,
    event::Key,
};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::style::Color;
use ratatui::Terminal;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use sqlx::SqlitePool;
use tracing::error;

pub enum Focus {
    Host,
    Filter,
    Proc,
    TotalProc,
    File,
}
pub struct App {
    focus: Focus,
    pool: SqlitePool,
    help: HelpComponent,
    pub error: ErrorComponent,
    pub config: KeyConfig,
    pub host: HostInfoComponent,
    pub footer: FooterComponent,
    pub query_input: QueryInputComponents,
    pub proc: ProcInfoComponent,
    pub total_proc: TotalProcInfoComponent,
    pub tabs: TabComponent,
    pub file: SyntaxTextComponent,
}

impl App {
    pub async fn new(config: KeyConfig, db: &SqlitePool) -> anyhow::Result<App> {
        let proc_com: ProcInfoComponent;
        let total_com: TotalProcInfoComponent =
            TotalProcInfoComponent::new(db, config.clone()).await?;
        let host_com: HostInfoComponent;
        match ProcInfoComponent::new(db, config.clone()).await {
            Ok(p) => proc_com = p,
            Err(e) => {
                error!("error: {:#?}", e);
                std::process::exit(1);
            }
        }
        match HostInfoComponent::new(db, config.clone()).await {
            Ok(h) => host_com = h,
            Err(e) => {
                error!("error: {:#?}", e);
                std::process::exit(1);
            }
        }
        Ok(Self {
            error: ErrorComponent::new(config.clone()),
            config: config.clone(),
            focus: Focus::Host,
            help: HelpComponent::new(config.clone()),
            pool: db.clone(),
            host: host_com,
            proc: proc_com,
            total_proc: total_com,
            footer: FooterComponent {
                colors: TableColors::new(&PALETTES[0]),
            },
            query_input: QueryInputComponents::new(config.clone()),
            tabs: TabComponent::new(config.clone()),
            file: SyntaxTextComponent::new(config.clone()),
        })
    }

    pub fn draw(&mut self, f: &mut Frame) -> anyhow::Result<()> {
        match self.tabs.selected_tab {
            SelectedTab::Tab2 => match self.focus {
                Focus::File => {
                    self.file.draw(f, f.size(), false)?;
                    self.error.draw(f, Rect::default(), false)?;
                    self.help.draw(f, Rect::default(), false)?;
                }
                _ => {
                    let rects = Layout::vertical([
                        Constraint::Length(4),
                        Constraint::Length(5),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ])
                    .split(f.size());
                    self.tabs.draw(f, rects[0], false)?;
                    self.query_input.draw(f, rects[1], false)?;
                    if self.query_input.history.len() > 0 {
                        self.total_proc.draw(f, rects[2], false)?;
                    } else {
                        self.proc.draw(f, rects[2], false)?;
                    }
                    self.footer.draw(f, rects[3], false)?;
                    self.error.draw(f, Rect::default(), false)?;
                }
            },
            SelectedTab::Tab1 => {
                let rects = Layout::vertical([
                    Constraint::Length(4),
                    Constraint::Length(5),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(f.size());

                self.tabs.draw(f, rects[0], false)?;
                self.query_input.draw(f, rects[1], false)?;
                self.host.draw(f, rects[2], false)?;
                self.footer.draw(f, rects[3], false)?;
                self.error.draw(f, Rect::default(), false)?;
                self.help.draw(f, Rect::default(), false)?;
            }
        }

        Ok(())
    }

    fn update_commands(&mut self) {
        self.help.set_cmds(self.commands());
    }

    fn commands(&self) -> Vec<CommandInfo> {
        let mut res = vec![
            CommandInfo::new(command::exit_pop_up(&self.config)),
            CommandInfo::new(command::filter(&self.config)),
            CommandInfo::new(command::help(&self.config)),
            CommandInfo::new(command::toggle_tabs(&self.config)),
            // CommandInfo::new(command::scroll(&self.config)),
            CommandInfo::new(command::scroll_to_top_bottom(&self.config)),
            CommandInfo::new(command::scroll_up_down_multiple_lines(&self.config)),
            CommandInfo::new(command::move_focus(&self.config)),
            CommandInfo::new(command::extend_or_shorten_widget_width(&self.config)),
        ];

        self.host.commands(&mut res);
        self.help.commands(&mut res);
        res
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        self.update_commands();

        if self.components_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };
        if self.move_focus(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        Ok(EventState::NotConsumed)
    }

    async fn components_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.error.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        if self.help.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::Host => {
                let state = self.host.event(key)?;
                return Ok(state);
            }
            Focus::Filter => match self.tabs.selected_tab {
                SelectedTab::Tab1 => {
                    let state = self.query_input.event(key)?;
                    if self.query_input.history.len() > 0 && key == Key::Enter {
                        let query_ast = self.query_input.history.first().unwrap().clone();
                        match self.query_input.query_type {
                            QueryType::Tcm => {
                                self.select_target_host_by_tcm(query_ast).await?;
                            }
                            QueryType::Text => {
                                let complate_hosts = select_all_host(&self.pool).await?;
                                self.host.items = search_vec(&complate_hosts, &query_ast);
                            }
                        }
                        return Ok(EventState::Consumed);
                    }
                    if key == Key::Up || key == Key::Down {
                        self.reset_focus_data().await?
                    }
                    return Ok(state);
                }
                SelectedTab::Tab2 => {
                    let state = self.query_input.event(key)?;
                    if self.query_input.history.len() > 0 && key == Key::Enter {
                        let query_ast = self.query_input.history.first().unwrap().clone();
                        match self.query_input.query_type {
                            QueryType::Tcm => {
                                let all_proc = self.select_target_proc_by_tcm(&query_ast).await?;
                                self.total_proc.items = all_proc;
                            }
                            QueryType::Text => {
                                let all_proc = self.select_target_proc_by_tcm("*.*.*.*").await?;
                                self.total_proc.items = search_vec(&all_proc, &query_ast);
                            }
                        }
                    }
                    if key == Key::Up || key == Key::Down {
                        self.reset_focus_data().await?
                    }
                    return Ok(state);
                }
            },
            Focus::Proc => {
                if let Some(item) = &self.proc.select_item {
                    match key {
                        Key::Enter => {
                            let path = std::path::Path::new(&item.work_path);
                            let p = path.join(&item.funcname);
                            self.file.load_file(&p)?;
                            self.file.show()?;
                            self.focus = Focus::File;
                            return Ok(EventState::Consumed);
                        }
                        _ => {
                            let state = self.proc.event(key)?;
                            return Ok(state);
                        }
                    }
                }
                let state = self.proc.event(key)?;
                return Ok(state);
            }
            Focus::TotalProc => {
                if let Some(item) = &self.total_proc.select_item {
                    match key {
                        Key::Enter => {
                            let path = std::path::Path::new(&item.work_path);
                            let p = path.join(&item.proc_name);
                            self.file.load_file(&p)?;
                            self.file.show()?;
                            self.focus = Focus::File;
                            return Ok(EventState::Consumed);
                        }
                        _ => {
                            let state = self.total_proc.event(key)?;
                            return Ok(state);
                        }
                    }
                }
                let state = self.total_proc.event(key)?;
                return Ok(state);
            }
            Focus::File => match key {
                Key::Esc => {
                    self.file.clear();
                    self.file.hide();
                    self.focus = Focus::TotalProc;
                    return Ok(EventState::Consumed);
                }
                _ => {
                    let state = self.file.event(key)?;
                    return Ok(state);
                }
            },
        }
    }

    async fn reset_focus_data(&mut self) -> anyhow::Result<()> {
        self.host.items = select_all_host(&self.pool).await?;
        self.query_input.history = vec![];
        Ok(())
    }

    async fn move_focus(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.tabs.selected_tab {
            crate::components::tabs::SelectedTab::Tab1 => {
                if key == Key::Char('2') {
                    self.query_input.tab = SelectedTab::Tab2;
                    self.focus = Focus::Proc;
                    self.query_input.history = vec![];
                    self.tabs.selected_tab = SelectedTab::Tab2;
                    return Ok(EventState::Consumed);
                }
                match self.focus {
                    Focus::Filter => {
                        if key == Key::Tab {
                            self.query_input.color = Color::Reset;
                            self.focus = Focus::Host;
                            return Ok(EventState::Consumed);
                        }
                    }
                    Focus::Host => {
                        if key == Key::Tab {
                            self.focus = Focus::Filter;
                            self.query_input.color = Color::Red;
                            return Ok(EventState::Consumed);
                        }
                    } // _ => return Ok(EventState::Consumed),
                    _ => {}
                }
            }
            crate::components::tabs::SelectedTab::Tab2 => {
                if key == Key::Char('1') {
                    self.query_input.tab = SelectedTab::Tab1;
                    self.focus = Focus::Host;
                    self.query_input.history = vec![];
                    self.tabs.selected_tab = SelectedTab::Tab1;
                    return Ok(EventState::Consumed);
                }
                match self.focus {
                    Focus::Filter => {
                        if key == Key::Tab {
                            if self.query_input.history.len() > 0 {
                                self.focus = Focus::TotalProc;
                                self.query_input.color = Color::Reset;
                                return Ok(EventState::Consumed);
                            } else {
                                self.focus = Focus::Proc;
                                self.query_input.color = Color::Reset;
                                return Ok(EventState::Consumed);
                            }
                        }
                    }
                    Focus::TotalProc => {
                        if key == Key::Tab {
                            self.focus = Focus::Filter;
                            self.query_input.color = Color::Red;
                            return Ok(EventState::Consumed);
                        }
                    }
                    Focus::Proc => {
                        if key == Key::Tab {
                            self.focus = Focus::Filter;
                            self.query_input.color = Color::Red;
                            return Ok(EventState::Consumed);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(EventState::NotConsumed)
    }
    async fn select_target_host_by_tcm(&mut self, tcm_sql: String) -> anyhow::Result<()> {
        let result = query_hosts_sql(&tcm_sql, &self.pool).await?;
        let hosts: Vec<HostInfo> = result.iter().map(|f| f.into()).collect::<Vec<HostInfo>>();
        self.host.items = hosts;
        Ok(())
    }
    async fn select_target_proc_by_tcm(&self, tcm_sql: &str) -> anyhow::Result<Vec<TotalProc>> {
        let result = query_hosts_sql(tcm_sql, &self.pool).await?;
        let procs: Vec<TotalProc> = result.iter().map(|f| f.into()).collect::<Vec<TotalProc>>();
        Ok(procs)
    }
}

pub async fn start_app(db: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let config = config::KeyConfig::default();
    let events = crate::event::Events::new(250);

    // create app and run it
    terminal.clear()?;
    let mut app = App::new(config, db).await?;
    loop {
        terminal.draw(|f| {
            if let Err(err) = app.draw(f) {
                println!("start draw terminal error {}", err);
                std::process::exit(1);
            }
        })?;
        match events.next()? {
            Event::Input(key) => match app.event(key).await {
                Ok(state) => {
                    // 这里是代表是否为耗时类型操作
                    if !state.is_consumed() && (key == app.config.quit || key == app.config.exit) {
                        break;
                    }
                }
                Err(err) => app.error.set(err.to_string())?,
            },
            Event::Tick => (),
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    drop_app();
    Ok(())
}

use super::command::CommandText;
use super::{CommandInfo, Component, DrawableComponent, EventState};
use crate::ui::scrollbar::{draw_scrollbar, Orientation};
use crate::ui::stateful_paragraph::{ParagraphState, ScrollPos, StatefulParagraph};
use crate::utils::{file_content, tabs_to_spaces};
use crate::{config::KeyConfig, event::Key};
use anyhow::Result;
use itertools::Either;
use once_cell::sync::Lazy;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Wrap},
    Frame,
};
use scopetime::scope_time;
use std::cell::Cell;
use std::ffi::OsStr;
use std::ops::Range;
use std::path::{Path, PathBuf};
use syntect::{
    highlighting::{
        FontStyle, HighlightState, Highlighter, RangedHighlightIterator, Style, ThemeSet,
    },
    parsing::{ParseState, ScopeStack, SyntaxSet},
};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(two_face::syntax::extra_no_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

#[derive(Copy, Clone, Debug)]
pub enum MoveSelection {
    Up,
    Down,
    Left,
    Right,
    Top,
    End,
    PageDown,
    PageUp,
}
pub fn scroll(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll [{}{}]",
            key_config.scroll_up, key_config.scroll_down
        ),
        "scroll up or down in focused view",
    )
}

pub struct SyntaxTextComponent {
    content: Option<(String, Either<SyntaxText, String>)>,
    key_config: KeyConfig,
    visible: bool,
    // file_path: Option<PathBuf>,
    paragraph_state: Cell<ParagraphState>,
    theme: Color,
}

impl SyntaxTextComponent {
    ///
    pub fn new(key: KeyConfig) -> Self {
        Self {
            content: None,
            key_config: key,
            visible: false,
            // file_path: None,
            theme: Color::LightMagenta,
            paragraph_state: Cell::new(ParagraphState::default()),
        }
    }

    pub fn clear(&mut self) {
        self.content = None
    }

    ///
    pub fn load_file(&mut self, path: &std::path::PathBuf) -> anyhow::Result<()> {
        let content = file_content(&path)?;
        let content = tabs_to_spaces(content);
        let p = Path::new(&path);
        let sy = SyntaxText::new(content, p)?;
        self.content = Some((path.to_str().unwrap().to_string(), Either::Left(sy)));
        Ok(())
    }

    fn scroll(&self, nav: MoveSelection) -> bool {
        let state = self.paragraph_state.get();

        let new_scroll_pos = match nav {
            MoveSelection::Down => state.scroll().y.saturating_add(1),
            MoveSelection::Up => state.scroll().y.saturating_sub(1),
            MoveSelection::Top => 0,
            MoveSelection::End => state
                .lines()
                .saturating_sub(state.height().saturating_sub(2)),
            MoveSelection::PageUp => state
                .scroll()
                .y
                .saturating_sub(state.height().saturating_sub(2)),
            MoveSelection::PageDown => state
                .scroll()
                .y
                .saturating_add(state.height().saturating_sub(2)),
            _ => state.scroll().y,
        };

        self.set_scroll(new_scroll_pos)
    }

    fn set_scroll(&self, pos: u16) -> bool {
        let mut state = self.paragraph_state.get();

        let new_scroll_pos = pos.min(
            state
                .lines()
                .saturating_sub(state.height().saturating_sub(2)),
        );

        if new_scroll_pos == state.scroll().y {
            return false;
        }

        state.set_scroll(ScrollPos {
            x: 0,
            y: new_scroll_pos,
        });
        self.paragraph_state.set(state);

        true
    }
}

impl DrawableComponent for SyntaxTextComponent {
    fn draw(&self, f: &mut Frame, area: Rect, _foced: bool) -> Result<()> {
        if self.visible {
            let text = self.content.as_ref().map_or_else(
                || Text::from(""),
                |(_, content)| match content {
                    Either::Left(syn) => syn.into(),
                    Either::Right(s) => Text::from(s.as_str()),
                },
            );

            let title = format!(
                "{}",
                self.content
                    .as_ref()
                    .map(|(name, _)| name.clone())
                    .unwrap_or_default(),
            );

            let content = StatefulParagraph::new(text)
                .wrap(Wrap { trim: false })
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(return_title_theme(self.visible)),
                );

            let mut state = self.paragraph_state.get();

            f.render_stateful_widget(content, area, &mut state);

            self.paragraph_state.set(state);

            self.set_scroll(state.scroll().y);

            if self.visible {
                draw_scrollbar(
                    f,
                    area,
                    &self.theme,
                    usize::from(
                        state
                            .lines()
                            .saturating_sub(state.height().saturating_sub(2)),
                    ),
                    usize::from(state.scroll().y),
                    Orientation::Vertical,
                );
            }
        }

        Ok(())
    }
}

impl Component for SyntaxTextComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(scroll(&self.key_config)))
    }

    fn event(&mut self, key: crate::event::Key) -> anyhow::Result<EventState> {
        let nva: MoveSelection;
        match key {
            Key::Char('k') | Key::Up => nva = MoveSelection::Up,
            Key::Char('j') | Key::Down => nva = MoveSelection::Down,
            Key::Char('l') | Key::Left => nva = MoveSelection::Left,
            Key::Char('h') | Key::Right => nva = MoveSelection::Right,
            Key::PageDown | Key::Ctrl('f') => nva = MoveSelection::PageDown,
            Key::PageUp | Key::Ctrl('b') => nva = MoveSelection::PageUp,
            Key::End => nva = MoveSelection::End,
            // Key::
            // MoveSelection::End => self.selection_end(selection),
            _ => nva = MoveSelection::Top,
        };
        if self.scroll(nva) {
            return Ok(EventState::Consumed);
        } else {
            return Ok(EventState::NotConsumed);
        }

    }

    ///
    fn hide(&mut self) {
        self.visible = false;
    }

    fn show(&mut self) -> Result<()> {
        self.visible = true;

        Ok(())
    }
}
#[derive(Debug)]
struct SyntaxLine {
    items: Vec<(Style, usize, Range<usize>)>,
}

#[derive(Debug)]
pub struct SyntaxText {
    text: String,
    lines: Vec<SyntaxLine>,
    path: PathBuf,
}

impl SyntaxText {
    pub fn new(text: String, file_path: &Path) -> anyhow::Result<Self> {
        scope_time!("syntax_highlighting");

        let mut state = {
            scope_time!("syntax_highlighting.0");
            let syntax = file_path.extension().and_then(OsStr::to_str).map_or_else(
                || SYNTAX_SET.find_syntax_by_path(file_path.to_str().unwrap_or_default()),
                |ext| SYNTAX_SET.find_syntax_by_extension(ext),
            );

            ParseState::new(syntax.unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text()))
        };

        let highlighter = Highlighter::new(&THEME_SET.themes["base16-eighties.dark"]);

        let mut syntax_lines: Vec<SyntaxLine> = Vec::new();

        let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());

        {
            // let total_count = text.lines().count();

            for (number, line) in text.lines().enumerate() {
                let ops = state.parse_line(line, &SYNTAX_SET)?;
                let iter = RangedHighlightIterator::new(
                    &mut highlight_state,
                    &ops[..],
                    line,
                    &highlighter,
                );

                syntax_lines.push(SyntaxLine {
                    items: iter
                        .map(|(style, _, range)| (style, number, range))
                        .collect(),
                });
            }
        }

        Ok(Self {
            text,
            lines: syntax_lines,
            path: file_path.into(),
        })
    }

    ///
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<'a> From<&'a SyntaxText> for ratatui::text::Text<'a> {
    fn from(v: &'a SyntaxText) -> Self {
        let mut result_lines: Vec<Line> = Vec::with_capacity(v.lines.len());

        for (syntax_line, line_content) in v.lines.iter().zip(v.text.lines()) {
            let mut line_span: Line = Vec::with_capacity(syntax_line.items.len()).into();

            for (style, _, range) in &syntax_line.items {
                let item_content = &line_content[range.clone()];
                let item_style = syntact_style_to_tui(style);

                line_span.spans.push(Span::styled(item_content, item_style));
            }

            result_lines.push(line_span);
        }

        result_lines.into()
    }
}

fn syntact_style_to_tui(style: &Style) -> ratatui::style::Style {
    let mut res = ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
    ));

    if style.font_style.contains(FontStyle::BOLD) {
        res = res.add_modifier(ratatui::style::Modifier::BOLD);
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        res = res.add_modifier(ratatui::style::Modifier::ITALIC);
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        res = res.add_modifier(ratatui::style::Modifier::UNDERLINED);
    }

    res
}

fn return_title_theme(focused: bool) -> ratatui::style::Style {
    if focused {
        ratatui::style::Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        ratatui::style::Style::default().fg(Color::DarkGray)
    }
}

use ratatui::{prelude::*, widgets::*};

use super::{host::TableColors, DrawableComponent};
const INFO_TEXT: &str =
    "(Tab) change Tab | (enter) search | (q) quit | (↑) move up | (↓) move down | (1) host search | (2) proc search";


#[derive(Clone)]
pub struct FooterComponent {
    pub colors: TableColors,
}

impl DrawableComponent for FooterComponent {
    fn draw(&self, f: &mut Frame, _area: Rect, _focused: bool) -> anyhow::Result<()> {
        // let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());
        let mut s = self.clone();

        // s.set_colors();

        s.render_footer(f, _area);
        Ok(())
    }
}
impl FooterComponent {
    fn render_footer(&mut self, f: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Line::from(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(self.colors.footer_border_color))
                    .border_type(BorderType::Double),
            );
        f.render_widget(info_footer, area);
    }
}

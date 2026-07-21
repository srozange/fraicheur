//! Rendu de l'interface ratatui.

use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let [input_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let [list_area, preview_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Min(0)]).areas(body_area);

    render_input(frame, app, input_area);
    render_list(frame, app, list_area);
    render_preview(frame, app, preview_area);
    render_footer(frame, app, footer_area);
}

fn render_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let input = Paragraph::new(app.query.as_str())
        .block(Block::bordered().title(" Recherche "))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, area);
    // Curseur en fin de requête (largeur en cellules ~ nb de chars).
    let cursor_x = area.x + 1 + app.query.chars().count() as u16;
    frame.set_cursor_position(Position::new(cursor_x, area.y + 1));
}

fn render_list(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .filter_map(|&i| app.projects.get(i))
        .map(|p| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}/", p.workspace),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(p.name.clone(), Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let title = format!(" Projets ({}) ", app.filtered.len());
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_preview(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (title, text) = match app.selected_project() {
        Some(p) => (
            format!(" {} ", p.path.display()),
            if p.preview.is_empty() {
                "(aucun README / CLAUDE.md / AGENT.md)".to_string()
            } else {
                p.preview.clone()
            },
        ),
        None => (" Preview ".to_string(), "Aucun résultat.".to_string()),
    };

    let preview = Paragraph::new(text)
        .block(Block::bordered().title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, area);
}

fn render_footer(frame: &mut Frame, _app: &App, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::raw(" naviguer  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" cd  "),
        Span::styled("Ctrl+R", Style::default().fg(Color::Cyan)),
        Span::raw(" cd + claude --resume  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" annuler"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}

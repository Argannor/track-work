use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, List, ListItem,
    },
};
use tui::layout::Direction;
use tui::style::Color;
use tui::widgets::{ListState, Paragraph, Wrap};

use crate::app::{App, Focus, Mode};
use crate::log::LOG;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // let chunks = Layout::default()
    //     .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
    //     .split(f.size());
    draw_first_tab(f, app, f.size())
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{

    // Create a layout like this
    // -----------------------
    // |        row[0]       |
    // -----------------------
    // |          |          |
    // |  col[0]  |  col[1]  |
    // |          |          |
    // -----------------------

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5),
                Constraint::Percentage(100)
            ]
        )
        .split(area);

    draw_header(f, app, rows[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(50),
                Constraint::Percentage(90),
            ]
        )
        .split(rows[1]);

    draw_projects(f, app, columns[0]);
    draw_log(f, app, columns[1]);
}


fn draw_header<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let hotkey = Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD);

    let text = vec![
        Spans::from(format!("{:?}", app.mode)),
        match app.mode {
            Mode::NORMAL(_) => Spans::from(vec![
                Span::styled("/", hotkey),
                Span::raw(" filter mode    "),
                Span::styled("⏎", hotkey),
                Span::raw(" select    "),
                Span::styled("q", hotkey),
                Span::raw(" quit     "),
                Span::styled("p", hotkey),
                Span::raw(" pause 𝄽     "),
                Span::styled("r", hotkey),
                Span::raw(" resume ♪     "),
                Span::styled("s", hotkey),
                Span::raw(" stop ✓     "),
            ]),
            Mode::FILTER(_) => Spans::from(vec![
                Span::styled("⏎", hotkey),
                Span::raw(" normal mode    "),
                // Span::styled("<q>", hotkey),
                // Span::raw(" Quit     "),
            ])
        },
        Spans::from(""),
        if let Some(ref selected) = app.active_project {
            Spans::from(vec![
                Span::raw(format!("{}", selected))
            ])
        } else {
            Spans::from("")
        },
    ];
    let block = Block::default().borders(Borders::NONE);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}


fn draw_projects<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    // Draw projects
    let projects: Vec<ListItem> = app
        .projects
        .items_filtered
        .iter()
        .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
        .collect();
    let title_color = match (&app.mode, &app.focus) {
        (Mode::FILTER(_), Focus::PROJECTS) => Color::LightCyan,
        _ => Color::White,
    };
    let title: String = if app.projects.filter.len() > 0 {
        format!(" Projects (filter: {}) ", app.projects.filter)
    } else {
        " Projects ".to_string()
    };
    let title = Span::styled(title, Style::default().fg(title_color));
    let mut block = Block::default().borders(Borders::ALL).title(title);
    if app.focus == Focus::PROJECTS {
        block = block.border_style(Style::default().fg(Color::Green));
    }
    let projects = List::new(projects)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    // println!(">> {:?}", app.projects.state.selected());
    f.render_stateful_widget(projects, area, &mut app.projects.state);
    // println!("<< {:?}", app.projects.state.selected());
}

fn draw_log<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{

    let title_color = match (&app.mode, &app.focus) {
        (Mode::FILTER(_), Focus::PROJECTS) => Color::LightCyan,
        _ => Color::White,
    };
    let title: String = if app.projects.filter.len() > 0 {
        format!(" Log (filter: {}) ", app.projects.filter)
    } else {
        " Log ".to_string()
    };

    let mut block = Block::default().borders(Borders::ALL)
        .title(Span::styled(title, Style::default().fg(title_color)));
    if app.focus == Focus::LOG {
        block = block.border_style(Style::default().fg(Color::Green));
    }

    // the area needs 2 lines for the block's borders.
    if area.height > 2 {
        let messages: Vec<ListItem> = LOG.lock().unwrap()
            .last_n(area.height as usize - 2)
            .iter()
            .map(|x| x.clone())
            .map(|x| ListItem::new(vec![Spans::from(Span::raw(x))]))
            .collect();
        let log = List::new(messages)
            .block(block);
        f.render_stateful_widget(log, area, &mut ListState::default())
    }
}

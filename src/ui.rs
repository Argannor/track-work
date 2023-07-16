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
use tui::layout::{Direction, Margin};
use tui::style::Color;

use tui::widgets::{Clear, ListState, Paragraph, Table, Wrap};


use crate::app::{App, Focus, Mode};

use crate::log::LOG;
use crate::widgets::week_picker::{WeekPicker};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    draw_screen(f, app, f.size());
}

fn draw_screen<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
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

    if app.focus == Focus::Report {
        draw_report(f, app, rows[1])
    }
}

fn draw_report<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let inner = area.inner(&Margin { vertical: 10, horizontal: 10 });


    let rows = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Percentage(100)
    ]).split(inner.inner(&Margin{vertical:0, horizontal: 1}));

    // TODO: create Popover Widget
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(0x11, 0x11, 0x15)));
    let paragraph = Paragraph::new(Span::from("Report"))
        .block(block)
        .wrap(Wrap { trim: true });

    let picker = WeekPicker{};
    f.render_widget(Clear, inner);
    f.render_widget(paragraph, inner);
    f.render_stateful_widget(picker, rows[1], &mut app.report.weekpicker);

    if let Some(report) = &app.report.report {
        let record_rows: Vec<tui::widgets::Row> = report.rows.iter().map(|x| x.into()).collect();
        let table: Table = Table::new(record_rows).widths(&[Constraint::Length(20), Constraint::Length(20)]);
        f.render_widget(table, rows[2])
    }
}

fn draw_header<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let hotkey = Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD);

    let text = vec![
        Spans::from(format!("{:?}", app.mode)),
        match app.mode {
            Mode::Normal(_) => Spans::from(vec![
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
                Span::styled("x", hotkey),
                Span::raw(" report     "),
                Span::styled("a", hotkey),
                Span::raw(format!(" {} auto switch     ", if app.auto_switch { "disable"} else { "enable" })),
            ]),
            Mode::Filter(_) => Spans::from(vec![
                Span::styled("⏎", hotkey),
                Span::raw(" normal mode    "),
            ])
        },
        Spans::from(""),
        if let Some(ref selected) = app.active_project {
            Spans::from(vec![
                Span::raw(format!("{selected}"))
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
        (Mode::Filter(_), Focus::Projects) => Color::LightCyan,
        _ => Color::White,
    };
    let title: String = if app.projects.filter.is_empty() {
        " Projects ".to_string()
    } else {
        format!(" Projects (filter: {}) ", app.projects.filter)
    };
    let title = Span::styled(title, Style::default().fg(title_color));
    let mut block = Block::default().borders(Borders::ALL).title(title);
    if app.focus == Focus::Projects {
        block = block.border_style(Style::default().fg(Color::Green));
    }
    let projects = List::new(projects)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(projects, area, &mut app.projects.state);
}

fn draw_log<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let title_color = match (&app.mode, &app.focus) {
        (Mode::Filter(_), Focus::Projects) => Color::LightCyan,
        _ => Color::White,
    };
    let title: String = if app.projects.filter.is_empty() {
        " Log ".to_string()
    } else {
        format!(" Log (filter: {}) ", app.projects.filter)
    };

    let mut block = Block::default().borders(Borders::ALL)
        .title(Span::styled(title, Style::default().fg(title_color)));
    if app.focus == Focus::Log {
        block = block.border_style(Style::default().fg(Color::Green));
    }

    // the area needs 2 lines for the block's borders.
    if area.height > 2 {
        let messages: Vec<ListItem> = LOG.lock().unwrap()
            .last_n(area.height as usize - 2)
            .into_iter()
            .map(|x| ListItem::new(vec![Spans::from(Span::raw(x))]))
            .collect();
        let log = List::new(messages)
            .block(block);
        f.render_stateful_widget(log, area, &mut ListState::default());
    }
}

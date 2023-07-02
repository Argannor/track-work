use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::ProjectState::WORKING;
use crate::input::filter_mode::FilterMode;
use crate::input::input_handler::InputHandler;
use crate::input::normal_mode::NormalMode;
use crate::log::{LOG, log};
use crate::repository::repository::WorkRecordRepository;
use crate::widgets::list::StatefulList;

const PROJECTS: [&str; 6] = [
    "XO", "EKS", "Xorcery", "4", "5", "Xylophon"
];

const WORK_RECORD_REPO: Lazy<Mutex<WorkRecordRepository>> = Lazy::new(||
    Mutex::new(WorkRecordRepository::new(
        env::current_dir().expect("cwd is not set")
            .into_os_string().into_string()
            .expect("could not convert cwd to string")
    ).expect("could not create database")));

pub trait Focusable {
    fn on_input(&mut self, event: KeyEvent);
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Mode {
    NORMAL(NormalMode),
    FILTER(FilterMode),
}

#[derive(PartialEq, Debug)]
pub enum Focus {
    PROJECTS,
    LOG,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TimeKind {
    PRODUCTIVE,
    PAUSE
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSegment {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub kind: TimeKind
}

impl TimeSegment {
    pub fn finish(&mut self) {
        if self.end.is_none() {
            self.end = Some(Utc::now());
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectState {
    WORKING,
    PAUSED,
    DONE,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveProject {
    pub id: String,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub state: ProjectState,
    pub segments: Vec<TimeSegment>
}

impl Drop for ActiveProject {
    fn drop(&mut self) {
        // FIXME: causes stack overflow. We need to split the ActiveProject (Behaviour) and Project (DTO) to prevent this
        self.stop()
    }
}

impl ActiveProject {

    pub fn load_previous() -> Option<ActiveProject> {
        WORK_RECORD_REPO.lock().unwrap().get_latest()
    }

    pub fn new(name: String) -> ActiveProject {
        let name: String = name.clone();
        let start = Utc::now();
        let project = ActiveProject {
            id: Uuid::new_v4().to_string(),
            name,
            start,
            end: None,
            state: WORKING,
            segments: vec![TimeSegment{
                start,
                end: None,
                kind: TimeKind::PRODUCTIVE
            }],
        };
        log!("{}", project);
        if let Err(e) = WORK_RECORD_REPO.lock().unwrap().persist(&project) {
            log!("failed to save work record for {}: {}", project.name, e);
        }

        project
    }

    pub fn begin_pause(&mut self) {
        self.state = ProjectState::PAUSED;
        if let Some(last_segment) = self.segments.last_mut() {
            last_segment.finish();
        }
        self.segments.push(TimeSegment{
            start: Utc::now(),
            end: None,
            kind: TimeKind::PAUSE
        });
    }

    pub fn resume_work(&mut self) {
        self.state = ProjectState::WORKING;
        if let Some(last_segment) = self.segments.last_mut() {
            last_segment.finish();
        }
        self.segments.push(TimeSegment{
            start: Utc::now(),
            end: None,
            kind: TimeKind::PRODUCTIVE
        });
    }

    pub fn stop(&mut self) {
        self.state = ProjectState::DONE;
        if let Some(last_segment) = self.segments.last_mut() {
            last_segment.finish();
            self.end = last_segment.end;
        }
        log!("{}", self);
        if let Err(e) = WORK_RECORD_REPO.lock().unwrap().persist(&self) {
            log!("failed to save work record for {}: {}", self.name, e);
        }
    }

    fn calculate_duration(&self) -> chrono::Duration {
        self.segments.iter()
            .filter(|segment| segment.kind == TimeKind::PRODUCTIVE)
            .map(|segment| {
                if let Some(end) = segment.end {
                    end.signed_duration_since(segment.start)
                } else {
                    Utc::now().signed_duration_since(segment.start)
                }
            })
            .reduce(|a,b| a.add(b))
            .expect("There should always be at least one segment to calculate a Duration")
    }
}

impl Display for ActiveProject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let icon = match self.state {
            ProjectState::WORKING => "♪",
            ProjectState::PAUSED => "𝄽",
            ProjectState::DONE => "✓"
        };

        let mut result = format!(
            "{} {}: {}",
            icon,
            self.name,
            self.start.with_timezone(chrono::Local::now().offset()).format("%Y-%m-%d %H:%M"));

        let duration = self.calculate_duration();
        if let Some(end) = self.end {
            result = format!("{} - {} (working time: {:02}:{:02}:{:02})",
                             result,
                             end.with_timezone(chrono::Local::now().offset()).format("%H:%M"),
                             duration.num_hours(), duration.num_minutes() % 60, duration.num_seconds() % 60
            );
        } else {
            result = format!("{} (working time: {:02}:{:02}:{:02})",
                             result,
                             duration.num_hours(), duration.num_minutes() % 60, duration.num_seconds() % 60
            );
        }

        f.write_str(&result)
    }
}


pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub projects: StatefulList<&'a str>,
    pub enhanced_graphics: bool,
    pub focus: Focus,
    pub mode: Mode,
    pub active_project: Option<ActiveProject>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        App {
            title,
            should_quit: false,
            projects: StatefulList::with_items(PROJECTS.to_vec()),
            focus: Focus::PROJECTS,
            mode: Mode::NORMAL(NormalMode {}),
            enhanced_graphics,
            active_project: ActiveProject::load_previous(),
        }
    }

    pub fn on_up(&mut self) {
        self.projects.previous();
    }

    pub fn on_down(&mut self) {
        self.projects.next();
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }

    fn normal_mode(&mut self) {
        self.set_mode(Mode::NORMAL(NormalMode {}))
    }

    fn filter_mode(&mut self) {
        self.set_mode(Mode::FILTER(FilterMode {}))
    }

    fn focus_next(&mut self) {
        self.focus = match self.focus {
            Focus::PROJECTS => Focus::LOG,
            Focus::LOG => Focus::PROJECTS,
        }
    }

    fn focus_previous(&mut self) {
        self.focus = match self.focus {
            Focus::PROJECTS => Focus::LOG,
            Focus::LOG => Focus::PROJECTS,
        }
    }

    pub fn start_working_on(&mut self, project: String) {
        if let Some(ref mut current_project) = self.active_project {
            current_project.end = Some(Utc::now());
            current_project.state = ProjectState::DONE;
            // todo safe previous
        }
        self.active_project = Some(ActiveProject::new(project));
    }

    pub fn on_input(&mut self, event: KeyEvent) {
        match (self.mode, event.code, event.kind) {
            (Mode::NORMAL(_), KeyCode::Char('/'), KeyEventKind::Release) => self.filter_mode(),
            (Mode::NORMAL(_), KeyCode::Char('q'), KeyEventKind::Release) => self.should_quit = true,
            (Mode::FILTER(_), KeyCode::Enter | KeyCode::Esc, KeyEventKind::Release) => self.normal_mode(),

            (_, KeyCode::Up, KeyEventKind::Press | KeyEventKind::Repeat) => self.on_up(),
            (_, KeyCode::Down, KeyEventKind::Press | KeyEventKind::Repeat) => self.on_down(),

            (_, KeyCode::Right | KeyCode::Tab, KeyEventKind::Press | KeyEventKind::Repeat) => self.focus_next(),
            (_, KeyCode::Left, KeyEventKind::Press | KeyEventKind::Repeat) => self.focus_previous(),

            (Mode::NORMAL(ref mode), _, _) => mode.on_input(event, self),
            (Mode::FILTER(ref mode), _, _) => mode.on_input(event, self),
        }
    }

    pub fn on_tick(&mut self) {
        // Update progress
    }

    pub fn get_focus(&mut self) -> Option<&mut dyn Focusable> {
        match self.focus {
            Focus::PROJECTS => Some(&mut self.projects),
            _ => None
        }
    }

    pub fn send_input_to_focus(&mut self, event: KeyEvent) {
        match self.focus {
            Focus::PROJECTS => self.projects.on_input(event),
            _ => {}
        };
        // if let Some(focus) = self.get_focus() {
        //     focus.on_input(self, event);
        // }
    }

    pub fn log(&self, fmt: String) {
        LOG.lock().unwrap().log(fmt);
        // log!("{}", fmt);
    }
}
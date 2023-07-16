use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::Mutex;

use chrono::{Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use once_cell::sync::Lazy;
use uuid::Uuid;

use crate::app::ProjectState::Working;
use crate::app_config::AppConfig;
use crate::input::filter_mode::FilterMode;
use crate::input::handler::InputHandler;
use crate::input::normal_mode::NormalMode;
use crate::log::{log};
use crate::repository::model::{ProjectState, TimeKind, TimeSegment, WorkRecord};
use crate::repository::work_record::WorkRecordRepository;
use crate::SETTINGS;
use crate::widgets::list::StatefulList;
use crate::report::Report;
use crate::widgets::week_picker::WeekPickerState;

static WORK_RECORD_REPO: Lazy<Mutex<WorkRecordRepository>> = Lazy::new(||
    Mutex::new(WorkRecordRepository::new(
        &env::current_dir().expect("cwd is not set")
            .into_os_string().into_string()
            .expect("could not convert cwd to string")
    ).expect("could not create database")));

pub trait Focusable {
    fn on_input(&mut self, event: &KeyEvent);
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Mode {
    Normal(NormalMode),
    Filter(FilterMode),
}

#[derive(PartialEq, Debug)]
pub enum Focus {
    Projects,
    Log,
    Report,
}

#[derive(Debug)]
pub struct ActiveProject {
    record: WorkRecord,
}

impl Drop for ActiveProject {
    fn drop(&mut self) {
        self.stop();
    }
}

impl ActiveProject {
    pub fn load_previous() -> Option<ActiveProject> {
        WORK_RECORD_REPO.lock().unwrap().get_latest().map(|record| ActiveProject { record })
    }

    pub fn new(name: String) -> ActiveProject {
        let start = Utc::now();
        let work_record = WorkRecord {
            id: Uuid::new_v4().to_string(),
            name,
            start,
            end: None,
            state: Working,
            segments: vec![TimeSegment {
                start,
                end: None,
                kind: TimeKind::Productive,
            }],
        };
        log!("{}", work_record);
        if let Err(e) = WORK_RECORD_REPO.lock().unwrap().persist(work_record.clone()) {
            log!("failed to save work record for {}: {}", work_record.name, e);
        }

        ActiveProject { record: work_record }
    }

    pub fn begin_pause(&mut self) {
        log!("𝄽 pausing work");
        self.record.state = ProjectState::Paused;
        if let Some(last_segment) = self.record.segments.last_mut() {
            last_segment.finish();
        }
        self.record.segments.push(TimeSegment {
            start: Utc::now(),
            end: None,
            kind: TimeKind::Pause,
        });
    }

    pub fn resume_work(&mut self) {
        log!("♪ resuming work");
        self.record.state = Working;
        if let Some(last_segment) = self.record.segments.last_mut() {
            last_segment.finish();
        }
        self.record.segments.push(TimeSegment {
            start: Utc::now(),
            end: None,
            kind: TimeKind::Productive,
        });
    }

    pub fn stop(&mut self) {
        if self.record.state == ProjectState::Done {
            return;
        }
        self.record.state = ProjectState::Done;
        if let Some(last_segment) = self.record.segments.last_mut() {
            last_segment.finish();
            self.record.end = last_segment.end;
        }
        log!("{}", self.record);
        if let Err(e) = WORK_RECORD_REPO.lock().unwrap().persist(self.record.clone()) {
            log!("failed to save work record for {}: {}", self.record.name, e);
        }
    }
}

impl Display for ActiveProject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.record, f)
    }
}

#[derive(Default, Debug)]
pub struct ReportState {
    pub weekpicker: WeekPickerState,
    pub report: Option<Report>,
}

impl ReportState {
    pub fn calculate(&mut self) {
        let (start, _) = self.weekpicker.start_and_end();
        let records = WORK_RECORD_REPO.lock().unwrap().find_week(&start);
        if let Ok(records) = records {
            self.report = Some(Report::new_pct(&records))
        } else {
            self.report = None;
        }
    }
}

#[derive(Debug)]
pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,

    pub mode: Mode,
    pub focus: Focus,
    pub auto_switch: bool,

    pub projects: StatefulList<&'a str>,
    pub active_project: Option<ActiveProject>,
    pub report: ReportState,

    config: AppConfig,
    auto_break: bool,
}

fn string_to_static_string<'a>(s: String) -> &'a str {
    Box::leak(s.into_boxed_str())
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> App<'a> {
        let config = SETTINGS.read().expect("could not acquire read lock on app settings");
        let config = config.deref();
        let projects: Vec<&'a str> = config
            .projects
            .iter()
            .map(|p| p.name.clone())
            .map(string_to_static_string)
            .collect();
        App {
            title,
            config: config.clone(),
            should_quit: false,
            projects: StatefulList::with_items(projects),
            focus: Focus::Projects,
            mode: Mode::Normal(NormalMode {}),
            active_project: ActiveProject::load_previous(),
            report: ReportState::default(),
            auto_break: false,
            auto_switch: true,
        }
    }

    pub fn on_up(&mut self) {
        self.projects.previous();
    }

    pub fn on_down(&mut self) {
        self.projects.next();
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn normal_mode(&mut self) {
        self.set_mode(Mode::Normal(NormalMode {}));
    }

    fn filter_mode(&mut self) {
        self.set_mode(Mode::Filter(FilterMode {}));
    }

    fn focus_next(&mut self) {
        self.focus = match self.focus {
            Focus::Projects => Focus::Log,
            Focus::Log => Focus::Projects,
            Focus::Report => Focus::Report
        };
    }

    fn focus_previous(&mut self) {
        self.focus = match self.focus {
            Focus::Projects => Focus::Log,
            Focus::Log => Focus::Projects,
            Focus::Report => Focus::Report
        };
    }

    pub fn start_working_on(&mut self, project: String) {
        if let Some(ref mut current_project) = self.active_project {
            if current_project.record.name == project {
                current_project.resume_work();
                return;
            }
            current_project.stop();
        }
        self.active_project = Some(ActiveProject::new(project));
    }

    pub fn on_input(&mut self, event: KeyEvent) {
        if self.focus == Focus::Report && self.on_report_input(event) {
            return;
        }
        match (self.mode, event.code, event.kind) {
            (Mode::Normal(_), KeyCode::Char('/'), KeyEventKind::Release) => self.filter_mode(),
            (Mode::Normal(_), KeyCode::Char('q'), KeyEventKind::Release) => self.should_quit = true,
            (Mode::Normal(_), KeyCode::Char('x'), KeyEventKind::Release) => self.focus = if self.focus == Focus::Report { Focus::Projects } else { Focus::Report },
            (Mode::Filter(_), KeyCode::Enter | KeyCode::Esc, KeyEventKind::Release) => self.normal_mode(),
            (_, KeyCode::Char('a'), KeyEventKind::Press | KeyEventKind::Repeat) => self.auto_switch = !self.auto_switch,

            (_, KeyCode::Up, KeyEventKind::Press | KeyEventKind::Repeat) => self.on_up(),
            (_, KeyCode::Down, KeyEventKind::Press | KeyEventKind::Repeat) => self.on_down(),

            (_, KeyCode::Right | KeyCode::Tab, KeyEventKind::Press | KeyEventKind::Repeat) => self.focus_next(),
            (_, KeyCode::Left, KeyEventKind::Press | KeyEventKind::Repeat) => self.focus_previous(),

            (Mode::Normal(ref mode), _, _) => mode.on_input(event, self),
            (Mode::Filter(ref mode), _, _) => mode.on_input(event, self),
        }
    }

    fn on_report_input(&mut self, event: KeyEvent) -> bool {
        let mut handled = true;
        match (event.code, event.kind) {
            (KeyCode::Left, KeyEventKind::Press | KeyEventKind::Repeat) => self.report.weekpicker.decrement(),
            (KeyCode::Right, KeyEventKind::Press | KeyEventKind::Repeat) => self.report.weekpicker.increment(),
            (KeyCode::Enter, KeyEventKind::Release) => self.report.calculate(),
            (KeyCode::Esc, KeyEventKind::Release) => self.focus = Focus::Projects,

            _ => handled = false
        }
        handled
    }

    pub(crate) fn on_window_focus_changed(&mut self, window_title: &String) {
        if !self.auto_switch {
            return;
        }
        if self.config.logging.window_change {
            log!("title changed: {}", window_title)
        }
        // if we previously went on auto break and auto resume is configured, resume
        if self.auto_break && self.config.breaks.auto_resume {
            if let Some(ref mut active_project) = self.active_project {
                active_project.resume_work();
            }
        }
        self.auto_break = false;

        // go over the list of projects and window titles to find a match (prefixes)
        let mut associated_project: Option<String> = None;
        for project in &self.config.projects {
            for window in &project.windows {
                if window_title.to_lowercase().starts_with(&window.to_lowercase()) {
                    associated_project = Some(project.name.clone());
                }
            }
        }
        // if a project was found, start work on that project
        if let Some(project) = associated_project {
            self.start_working_on(project);
            return;
        }

        // check if the window is configured to trigger an automatic break (i.e. lockscreens)
        let go_on_break = self.config.breaks.windows.iter()
            .any(|title| window_title.to_lowercase().starts_with(&title.to_lowercase()));
        if go_on_break {
            if let Some(ref mut active_project) = self.active_project {
                // start the break and set auto_break, so we can auto resume if configured
                active_project.begin_pause();
                self.auto_break = true;
            }
        }
    }

    pub fn on_tick(&mut self) {
        // Update progress
    }

    #[allow(dead_code)]
    pub fn get_focus(&mut self) -> Option<&mut dyn Focusable> {
        match self.focus {
            Focus::Projects => Some(&mut self.projects),
            Focus::Log => None,
            Focus::Report => None,
        }
    }

    pub fn send_input_to_focus(&mut self, event: &KeyEvent) {
        if self.focus == Focus::Projects {
            self.projects.on_input(event);
        }
    }
}
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::app::App;
use crate::input::handler::InputHandler;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct NormalMode {}

impl NormalMode {
    fn on_enter(self, app: &mut App) {
        {
            self.on_stop(app);
        }
        {
            if let Some(selection) = app.projects.get_selected() {
                app.start_working_on((*selection).to_string());
            }
        }
    }

    fn on_pause(self, app: &mut App) {
        if let Some(ref mut active_project) = app.active_project {
            active_project.begin_pause();
        }
    }
    fn on_resume(self, app: &mut App) {
        if let Some(ref mut active_project) = app.active_project {
            active_project.resume_work();
        }
    }
    fn on_stop(self, app: &mut App) {
        if let Some(ref mut active_project) = app.active_project {
            active_project.stop();
        }
    }
}

impl InputHandler for NormalMode {
    fn on_input(&self, event: KeyEvent, app: &mut App) {
        match (event.code, event.kind) {
            (KeyCode::Enter, KeyEventKind::Press) => self.on_enter(app),
            (KeyCode::Char('p'), KeyEventKind::Press) => self.on_pause(app),
            (KeyCode::Char('r'), KeyEventKind::Press) => self.on_resume(app),
            (KeyCode::Char('s'), KeyEventKind::Press) => self.on_stop(app),
            _ => {}
        }
    }
}

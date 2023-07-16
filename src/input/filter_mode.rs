use crossterm::event::{KeyEvent, KeyEventKind};

use crate::app::App;
use crate::input::handler::InputHandler;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct FilterMode {}

impl InputHandler for FilterMode {
    fn on_input(&self, event: KeyEvent, app: &mut App) {
        if event.kind == KeyEventKind::Press {
            app.send_input_to_focus(&event);
        }
    }
}

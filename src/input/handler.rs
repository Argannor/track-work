use crossterm::event::KeyEvent;

use crate::app::App;

pub trait InputHandler {
    fn on_input(&self, event: KeyEvent, app: &mut App);
}

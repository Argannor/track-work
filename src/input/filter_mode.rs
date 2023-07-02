use crossterm::event::{KeyEvent, KeyEventKind};

use crate::app::App;
use crate::input::input_handler::InputHandler;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct FilterMode {

}

impl FilterMode {
}

impl InputHandler for FilterMode {
    fn on_input(&self, event: KeyEvent, app: &mut App) {
        if event.kind == KeyEventKind::Press {
            app.send_input_to_focus(event);
        }
        // let mut focus = None;
        // {
        //     focus = app.get_focus();
        // }
        // if let Some(f) = focus {
        //     f.on_input(app, event)
        // }
        // app.log(format!("received {:?} ({:?})", event.code, event.kind));
    }
}

use crossterm::event::{KeyEvent, MouseEvent};



/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum AppOwnEvent {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Init,
    Error,
    Render,
    AllPassengersAlighted(u8),
    Exit
}


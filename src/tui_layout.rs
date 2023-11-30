use std::rc::Rc;

use ratatui::{layout::{Rect, Layout, Direction, Constraint},Terminal, backend::Backend};

use crate::app::AppResult;



#[derive(Debug)]
pub struct TuiLayout {
    pub display_windows: Rc<[Rect]>,
}

impl TuiLayout {

    pub fn  new<B: Backend>(terminal: &Terminal<B>) -> AppResult<TuiLayout> {

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref());
        let screen_chunks   = layout.split(terminal.size()?);
    
        Ok(TuiLayout { display_windows: screen_chunks })
    }

    pub fn get_window_corners(&self, windows_idx: i16) -> Rect {

        Rect {
            x: self.display_windows[windows_idx as usize].x,
            y: self.display_windows[windows_idx as usize].y,
            width: self.display_windows[windows_idx as usize].width,
            height: self.display_windows[windows_idx as usize].height
            
        }
    }

}


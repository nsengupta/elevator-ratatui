use std::rc::Rc;

use ratatui::{layout::{Rect, Layout, Direction, Constraint},Terminal, backend::Backend};

use crate::app::AppResult;



#[derive(Debug)]
pub struct TuiLayout {
    pub output_windows: Rc<[Rect]>,
    pub input_window: Rc<[Rect]>
}

impl TuiLayout {

    pub fn  new<B: Backend>(terminal: &Terminal<B>) -> AppResult<TuiLayout> {

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref());
        let screen_chunks_1   = layout.split(terminal.size()?);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref());
        let screen_chunks_2 = layout.split(screen_chunks_1[0]);
    
        Ok(TuiLayout { output_windows: screen_chunks_1, input_window: screen_chunks_2 })
    }

    pub fn get_window_corners(&self, windows_idx: i16) -> Rect {

        Rect {
            x: self.output_windows[windows_idx as usize].x,
            y: self.output_windows[windows_idx as usize].y,
            width: self.output_windows[windows_idx as usize].width,
            height: self.output_windows[windows_idx as usize].height
            
        }
    }

}


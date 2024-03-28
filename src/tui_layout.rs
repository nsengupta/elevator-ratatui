use std::rc::Rc;

use ratatui::{backend::Backend, layout::{Constraint, Direction, Layout, Rect},Terminal};
use tracing::info;

use crate::app::AppResult;

#[derive(Debug)]
pub struct TuiLayout {
    pub motion_window: Rc<[Rect]>,
    pub info_window: Rc<[Rect]>,
    pub button_windows: Rc<[Rect]>,
    pub motion_window_index: u16,
    pub info_window_index: u16,
    pub start_button_index: u16,
    pub stop_button_index: u16,
    pub current_floor_index: u16,
    pub next_stop_index: u16
    
}

impl TuiLayout {

    pub fn  new<B: Backend>(terminal: &Terminal<B>) -> AppResult<TuiLayout> {

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref());
        let screen_chunks_1   = layout.split(terminal.size()?);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref());
        let screen_chunks_2 = layout.split(screen_chunks_1[0]);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                    Constraint::Percentage(25), 
                    Constraint::Percentage(25), 
                    Constraint::Percentage(25),
                    Constraint::Percentage(25)
                    ].as_ref());
        let screen_chunks_3 = layout.split(screen_chunks_2[1]);
    
        Ok(TuiLayout {  motion_window: screen_chunks_1, 
                        info_window: screen_chunks_2,
                        button_windows: screen_chunks_3,
                        motion_window_index: 1,
                        info_window_index: 0,
                        start_button_index: 0,
                        stop_button_index: 1,
                        current_floor_index: 2,
                        next_stop_index: 3
                        
                 })
    }

    pub fn log_window_corners(&self) -> () {

        info!("Elevator Info window: {:?}",               self.info_window[0]);
        info!("Elevator Motion window: {:?}",             self.motion_window[1]);
        info!("Start Button window: {:?}",                self.button_windows[0]);
        info!("Stop Button window: {:?}",                 self.button_windows[1]);
        info!("Current Floor display window: {:?}",       self.button_windows[2]);
        info!("Destination Floor display window: {:?}",   self.button_windows[3]);
    }


}


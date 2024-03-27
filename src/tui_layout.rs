use std::rc::Rc;

use ratatui::{backend::Backend, buffer::{Buffer, Cell}, layout::{Constraint, Direction, Layout, Rect}, widgets::{Widget, WidgetRef}, Terminal};

use crate::app::AppResult;

#[derive(Debug)]
pub struct StartButtonWidget;

impl Widget for StartButtonWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
        self.render_ref(area, buf)
    }
}

impl WidgetRef for StartButtonWidget {
    
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        for x in area.left() + 1..area.right() - 1 {
            for y in area.top() + 1..area.bottom() - 1  {
                buf.get_mut(x, y)
                .set_fg(ratatui::style::Color::Green)
                .set_char('█');
            }
        }
    }
}


#[derive(Debug)]
pub struct StopButtonWidget;

impl Widget for StopButtonWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
        self.render_ref(area, buf)
    }
}

impl WidgetRef for StopButtonWidget {
    
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        for x in area.left() + 1..area.right() - 1 {
            for y in area.top() + 1..area.bottom() - 1 {
                buf.get_mut(x, y)
                .set_fg(ratatui::style::Color::Red)
                .set_char('█');
            }
        }
    }
}





#[derive(Debug)]
pub struct TuiLayout {
    pub output_windows: Rc<[Rect]>,
    pub input_window: Rc<[Rect]>,
    pub button_windows: Rc<[Rect]>,
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
    
        Ok(TuiLayout {  output_windows: screen_chunks_1, 
                        input_window: screen_chunks_2,
                        button_windows: screen_chunks_3,
                        start_button_index: 0,
                        stop_button_index: 1,
                        current_floor_index: 2,
                        next_stop_index: 3
                        
                 })
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


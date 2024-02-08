use std::error;
use crate::{machinery::CarriageActor, elevator_infra::{ElevatorInfra, FloorCoordinates}, async_event::AppOwnEvent};
use crossterm::event::{KeyEvent, KeyCode, MouseEvent};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    pub inner_display_setup: ElevatorInfra,
    inner_machinery: Option<CarriageActor>,
    to_quit: bool,
    pub tick_count: i32,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub last_tick_key_events: Vec<KeyEvent>,
    pub mouse_at: Vec<MouseEvent>

}

impl App {

    pub fn new (carriage_movement_area: ElevatorInfra,tick_rate: f64, frame_rate: f64) -> Self { 
        Self {
            inner_display_setup: carriage_movement_area,
            inner_machinery: None,
            to_quit: false,
            tick_count: 0i32,
            tick_rate,
            frame_rate,
            last_tick_key_events: Vec::new(),
            mouse_at: Vec::new()
        } 
    }

    pub fn quit(&mut self) -> () {
        self.to_quit = true;
        () 
    }

    pub fn floor_roof_to_reach(&self,floor_id: i16) -> Option<&FloorCoordinates> {

        self.inner_display_setup.floor_coords.get(floor_id as usize)

    }


    pub fn should_quit_app(&self) -> bool {
        self.to_quit
    }

    pub fn update_app(&mut self, app_event: AppOwnEvent) -> () {

        if let AppOwnEvent::Key(key) = app_event {
            match key.code {
              KeyCode::Char('q') => self.quit(),
              _ => {},
            }
          }
        else {
            match app_event {
                AppOwnEvent::Tick => {
                    self.tick_count = self.tick_count + 1;
                },
                AppOwnEvent::Mouse(m) => {
                    self.mouse_at.push(m);
                }
                _ => {}
            }
        }  
          ()
    }

    pub fn save_to_file(&self) -> () {

        for n in &self.mouse_at {
            println!(" Mouse-event kind {:?}, at Row{}:Column{}", n.kind, n.row, n.column);
        }
    }


}
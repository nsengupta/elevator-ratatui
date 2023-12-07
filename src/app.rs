use std::error;
use crate::{machinery::CarriageActor, elevator_infra::ElevatorInfra};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    pub inner_display_setup: ElevatorInfra,
    inner_machinery: Option<CarriageActor>,
    to_quit: bool
}

impl App {

    pub fn new (carriage_movement_area: ElevatorInfra) -> Self { 
        Self {
            inner_display_setup: carriage_movement_area,
            inner_machinery: None,
            to_quit: false
        } 
    }

    pub fn quit(&mut self) -> () {
        self.to_quit = true;
        () 
    }

    pub fn increment_counter(&mut self) -> () { () }

    pub fn decrement_counter(&mut self) -> () { () }

    pub fn should_quit_app(&self) -> bool {
        self.to_quit
    }


}
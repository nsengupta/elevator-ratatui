use std::error;
use crate::{machinery::CarriageActor, app_layout::CarriageParameters};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    inner_machinery: Option<CarriageActor>,
    pub inner_display_setup: CarriageParameters
}

impl App {

    pub fn new (carriage_movement_area: CarriageParameters) -> Self { 
        Self {
            inner_display_setup: carriage_movement_area,
            inner_machinery: None 
        } 
    }

    pub fn quit(&mut self) -> () { () }

    pub fn increment_counter(&mut self) -> () { () }

    pub fn decrement_counter(&mut self) -> () { () }


}
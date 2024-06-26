


use ratatui::{layout::{Margin, Position}, prelude::{Color, Rect}, widgets::canvas::Rectangle};
use tracing::info;

pub const MX_FLOORS: u16 = 8;

#[derive(Debug)]
pub struct CarriageBox {
    pub  bottom_left_x_offset_from_origin: f64,
    pub  bottom_left_y_offset_from_origin: f64,
    pub  width:  f64,
    pub  height: f64
}

impl CarriageBox {
    pub fn _move_up(&mut self, displacement: f64) -> &mut Self {
        self.bottom_left_y_offset_from_origin += displacement;
        self
    }

    pub fn _move_down(&mut self, displacement: f64) -> &mut Self {
        self.bottom_left_y_offset_from_origin -= displacement;
        self
    }

    pub fn move_to_ground(&mut self) -> &mut Self {
        self.bottom_left_y_offset_from_origin = 0.0;
        self
    }
}

#[derive(Debug)]
pub struct Separator {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x:   f64,
    pub end_y:   f64
}

#[derive(Debug)]
pub struct ElevatorVisualInfra {
    pub carriage_box: CarriageBox,
    pub carriage_playground: Rect,
    pub each_floor_height: u16,
    pub floor_as_rects: Vec<Rect>,
    pub floors_having_passengers: Vec<bool>,
    pub dest_floor: Option<u16>,
    pub current_floor: Option<u16>,
    show_carriage_box: bool,
    destination_reached: bool,

}

impl  ElevatorVisualInfra {
    pub(crate) fn new(movement_area: Rect) -> Self {

        let carriage_playground = movement_area.inner(&Margin{ horizontal: 1, vertical: 1});

        let each_floor_height = 
            f64::floor(
                (carriage_playground.height as f64) / MX_FLOORS as f64
            ) 
            as u16;

        //  Remains fixed across all floors
        //  Obviously, the bottom_left_x, will have the same value
        //  Leave a little space from left border
        let each_floor_top_left_x: u16 = carriage_playground.x; 

        //  Measured from the wall separating the tunnel and floors
        //  Leave a little space from the right border
        let each_floor_width = (carriage_playground.width as f64 / 2.0) as u16 ;  

        //  Obviously, every floor will have a different top_left_y
        let floor_specific_top_left_y: Vec<u16> = 
                        (0..MX_FLOORS)
                        .rev()
                        .into_iter()
                        .map(|next_floor| {
                            carriage_playground.y as u16 + (each_floor_height * next_floor)
                        })
                        .collect()
                        ;

        let all_floors_represented_as_rects: Vec<Rect> = 
                floor_specific_top_left_y.iter()
                .map(|next_floor_top_left_y| {
                   Rect{
                        x:       each_floor_top_left_x,
                        y:       *next_floor_top_left_y,
                        width:   each_floor_width,
                        height:  each_floor_height
                   }
                })
                .collect()
                ;        

        let carriage_box = CarriageBox {
            bottom_left_x_offset_from_origin:  carriage_playground.width as f64/2.0,
            bottom_left_y_offset_from_origin:  0.0,
            width:                             carriage_playground.width as f64/2.0,
            height:                            each_floor_height as f64
        };

        //info!("carriage box {:?}", carriage_box);

        let floors_having_passengers: Vec<bool> = vec![false; MX_FLOORS as usize];

        ElevatorVisualInfra {
            carriage_box,
            carriage_playground,
            each_floor_height,
            floor_as_rects: all_floors_represented_as_rects,
            floors_having_passengers: floors_having_passengers,
            show_carriage_box: false, // TODO: use a flag to indicate if elev is operation (Start/Stop)
            dest_floor: None,
            current_floor: None,
            destination_reached: false

        }
    }

    pub fn set_carriage_ready(&mut self) -> () {
        self.carriage_box.move_to_ground();
        self.show_carriage_box = true;
    }

    pub fn unset_carriage(&mut self) -> () {
        self.show_carriage_box =  false;
        self.current_floor = None;
    }

    pub fn should_show_carriage(&self) -> bool {
        self.show_carriage_box
    }

    pub fn get_carriage_displacement_map_per_floor(&self,origin: (u16,u16)) -> Vec<(f64,f64)> {
        self.floor_as_rects.iter().enumerate()
                .map(|(floor_index,rect)| {
                    let left_bottom_y = origin.1 + (floor_index as u16 * rect.height);
                    (0.0,left_bottom_y as f64)    // X coordinates for all floors remain unchanged
                })
                .collect()  
    }

    pub fn is_passenger_waiting_at_reachable_floor(&self,mouse_click_position: Position) -> Option<u16> {

        for next_floor in self.floor_as_rects.iter().enumerate() {
            if next_floor.1.contains(mouse_click_position) {
                return Some(next_floor.0 as u16);
            }     
        }

        None
    }

    pub fn serve_passenger_at(&mut self, at_floor: u16) -> () {
        self.floors_having_passengers[at_floor as usize] = true;
    }



    pub fn mark_floor_on_reaching_destination(&mut self, dest_floor: u16) -> () {
        self.floors_having_passengers[dest_floor as usize] = false;
    }

    pub fn set_next_destination(&mut self, to_floor: u16) {
        self.dest_floor = Some(to_floor);
    }

    pub fn on_reaching_destination(&mut self) -> () {
        self.destination_reached = true;
        self.current_floor = self.dest_floor;
        self.dest_floor = None;
    }

    pub fn on_carriage_moving_to(&mut self,  move_to: (f64,f64)) {
        self.carriage_box.bottom_left_y_offset_from_origin = move_to.1;
    }


}

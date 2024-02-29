


use ratatui::{layout::{Margin, Position}, prelude::{Color, Marker, Rect}, widgets::canvas::Rectangle};

pub const MX_FLOORS: u16 = 8;

#[derive(Debug)]
pub struct Separator {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x:   f64,
    pub end_y:   f64
}

#[derive(Debug)]
pub struct ElevatorInfra {
    pub carriage_shape: Rectangle,
    pub carriage_playground: Rect,
    pub marker: Marker,
    tick_count: i32,
    pub building_wall: Separator,
    pub each_floor_height: u16,
    pub floor_as_rects: Vec<Rect>,
    pub floors_having_passengers: Vec<bool>

}

impl  ElevatorInfra {
    pub(crate) fn new(movement_area: Rect) -> Self {

        let carriage_playground = movement_area.inner(&Margin{ horizontal: 1, vertical: 1});

        // Separator between the tunnel where the carriage moves and the floors where the passengers wait.
        let separator_marker_start_x = carriage_playground.left() as f64 + carriage_playground.width  as f64 / 2.0;
        let separator_marker_start_y = carriage_playground.top() as f64; // carriage_playground.y + carriage_playground.height;
        let separator_marker_end_x = separator_marker_start_x; // x doesn't change for the wall
        let separator_marker_end_y = carriage_playground.bottom() as f64; // Top most level on the playground

        // We need a line to display the separator on the screen.
        let separator = Separator {
                        start_x: separator_marker_start_x  as f64,  // x1
                        start_y: separator_marker_start_y as f64, // y1
                        end_x: separator_marker_end_x  as f64,   // x2
                        end_y: separator_marker_end_y  as f64,   // y2
        };

        let each_floor_height = 
            f64::floor(
                (carriage_playground.height as f64) / MX_FLOORS as f64
            ) 
            as u16;

        //  Remains fixed across all floors
        //  Obviously, the bottom_left_x, if defined, will have the same value
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

        let carriage_shape = Rectangle {
            x: separator.start_x + 1.0,
            y: separator.start_y,
            width: 10.0,
            height: each_floor_height as f64,
            color: Color::Yellow,
        };

        let floors_having_passengers: Vec<bool> = vec![false; MX_FLOORS as usize];

        ElevatorInfra {
            carriage_shape,
            carriage_playground,
            marker: Marker::Block,
            tick_count: 0,
            building_wall: separator,
            each_floor_height,
            floor_as_rects: all_floors_represented_as_rects,
            floors_having_passengers: floors_having_passengers

        }
    }

    pub fn tranlate_coords_to_viewport(&self,floor_index: usize, origin: (u16,u16)) -> Rectangle {
        let floor_rect = self.floor_as_rects[floor_index as usize];
        let passenger_at_floor_indicator = self.floors_having_passengers[floor_index];
        let left_top_y = origin.1 + (floor_index as u16 * floor_rect.height);
        Rectangle {
            x: origin.0 as f64,
            y: left_top_y as f64,
            width: floor_rect.width as f64,
            height: floor_rect.height as f64,
            color: if passenger_at_floor_indicator { Color::Red } else { Color::LightBlue }
        }
    }

    pub fn get_carriage_displacement_map_per_floor(&self,origin: (u16,u16)) -> Vec<(f64,f64)> {
        self.floor_as_rects.iter().enumerate()
                .map(|(floor_index,rect)| {
                    let left_bottom_y = origin.1 + (floor_index as u16 * rect.height);
                    let left_top_y    = left_bottom_y + rect.height;
                    (left_top_y as f64, left_bottom_y as f64) 
                })
                .collect()  
    }

    pub fn is_passenger_calling_to_reachable_floor(&self,mouse_click_position: Position) -> Option<u16> {

        for next_floor in self.floor_as_rects.iter().enumerate() {
            if next_floor.1.contains(mouse_click_position) {
                return Some(next_floor.0 as u16);
            }     
        }

        None
    }

    pub fn on_passenger_summoning(&mut self, at_floor: u16) -> () {

        self.floors_having_passengers[at_floor as usize] = true;
    }

    pub fn is_any_passenger_waiting(&self) -> Option<u16> {
        for (floor_no,is_waiting) in self.floors_having_passengers.iter().enumerate() {
            if *is_waiting { return Some (floor_no as u16)}
        }
        None
    }

    pub fn tell_me_more(&self) -> String {
        format!(
            "Carriage top-left-x({}),top-left-y({}),bot-right-x({}),bot_right-y({})) | Ground top-left-x({}),top-left-y({}),bot-right-x({}),bot_right-y({}))\n",
            self.carriage_shape.x,
            self.carriage_shape.y,
            self.carriage_shape.x+self.carriage_shape.width,
            self.carriage_shape.y+self.carriage_shape.height,
            self.carriage_playground.x,
            self.carriage_playground.y,
            self.carriage_playground.x + self.carriage_playground.width,
            self.carriage_playground.y + self.carriage_playground.height
        )
    }

    pub fn on_tick(&mut self,  move_by: (i16,i16)) {
        self.tick_count += 1;
        
       
        /* if self.ball.x < self.playground.left() as f64
            || self.ball.x + self.ball.width > self.playground.right() as f64
        {
            self.dir_x = !self.dir_x;
        } */

        let new_y_coord = self.carriage_shape.y + move_by.1 as f64;

        if new_y_coord > self.carriage_playground.top() as f64
            && new_y_coord + self.carriage_shape.height < self.carriage_playground.bottom() as f64
        {
            self.carriage_shape.y = new_y_coord // x remains unchanged
        }

        /* if self.dir_x {
            self.ball.x += self.vx;
        } else {
            self.ball.x -= self.vx;
        } */

        /* if self.dir_y {
            self.ball.y += self.vy;
        } else {
            self.ball.y -= self.vy
        } */
    }
}

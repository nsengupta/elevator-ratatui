use std::rc::Rc;
use ratatui::{widgets::canvas::{Rectangle, Line}, prelude::{Color, Marker, Rect}};

const MX_FLOORS: u16 = 7;

#[derive(Debug)]
pub struct XYPoint {
    x: f64,
    y: f64,
}

#[derive(Debug)]
pub struct FloorCoordinates {
    ground_left: XYPoint,
    ground_right: XYPoint,
    roof_left: XYPoint,
    roof_right: XYPoint
}

#[derive(Debug)]
pub struct ElevatorInfra {
    pub carriage_shape: Rectangle,
    pub carriage_playground: Rect,
    pub marker: Marker,
    tick_count: i32,
    dir_x: i16,
    dir_y: i16,
    pub building_wall: Line,
    // pub level_markers: Vec<Line>, // Assumption: 8 floors, 0 to 7
    pub each_floor_height: u16,
    // pub floor_coords: Vec<FloorCoordinates>,
    pub floor_as_rects: Vec<Rect>

}

impl  ElevatorInfra {
    pub(crate) fn new(movement_area: Rect) -> Self {

        let marker = Marker::Braille;
        let carriage_playground = Rect {
            x:      movement_area.left() + 1,
            y:      movement_area.top() + 1,
            width:  movement_area.width - 1,
            height: movement_area.height - 1
        };

        // Separator between the tunnel where the carriage moves and the floors where the passengers wait.
        let separator_marker_start_x = carriage_playground.width  as f64 / 2.0;
        let separator_marker_start_y = 1.0 as f64; // carriage_playground.y + carriage_playground.height;
        let separator_marker_end_x = separator_marker_start_x; // x doesn't change for the wall
        let separator_marker_end_y = carriage_playground.height as f64; // Top most level on the playground

        // We need a line to display the separator on the screen.
        let separator = Line::new(
                        separator_marker_start_x  as f64,  // x1
                        separator_marker_start_y  as f64, // y1
                        separator_marker_end_x  as f64,   // x2
                        separator_marker_end_y  as f64,   // y2
                        Color::Blue
                    );

        let each_floor_height = 
            f64::ceil(
                f64::abs(separator.y2 - separator.y1) / MX_FLOORS as f64
            ) 
            as u16;

        //  remains fixded across all floors, arbitrarily chosen, a little away from (0,0);
        //  obviously, the bottom_left_x, if defined, will have the same value
        let each_floor_top_left_x: u16 = 3; 

        let each_floor_width = separator_marker_end_x as u16 - each_floor_top_left_x;  

        //  Obviously, every floor will have a different top_left_y
        let all_floor_top_left_y: Vec<u16> = 
                        (0..MX_FLOORS).into_iter()
                        .map(|next_floor| {
                            separator.y1 as u16 + (each_floor_height * next_floor)
                        })
                        .collect()
                        ;

        let all_floors_represented_as_rects: Vec<Rect> = 
                all_floor_top_left_y.iter()
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

        /* let mut floor_ground_screen_position: Vec<(f64,f64)> = Vec::new();     
        for index in 0..MX_FLOORS {  
            floor_ground_screen_position.push((separator.x1, separator.y1 + (each_floor_height * index) as f64));
        }   
        
        let floor_screen_markers = 
                floor_ground_screen_position.iter()
                .map(|next_posn|{
                        Line::new(
                            3.0,  // Leave a space at the left
                            next_posn.1,
                            next_posn.0,
                            next_posn.1,
                            Color::Blue
                        )
                    })
                .collect::<Vec<Line>>()
                ;
                let floor_coords = 
                floor_screen_markers.iter()
                    .as_slice()
                    .windows(2)
                    .map(|next_pair| {
                      FloorCoordinates {
                        ground_left: XYPoint{ x: next_pair[0].x1, y: next_pair[0].y1 },
                        ground_right: XYPoint{ x: next_pair[0].x2,y: next_pair[0].y2 },
                        roof_left: XYPoint{ x: next_pair[1].x1, y: next_pair[1].y1 },
                        roof_right: XYPoint{ x: next_pair[1].x2, y: next_pair[1].y2 },
                      }
                    })
                    .collect::<Vec<FloorCoordinates>>()
                    ;
 */
        let carriage_shape = Rectangle {
            x: separator.x1 + 1.0,
            y: separator.y1,
            width: 10.0,
            height: each_floor_height as f64,
            color: Color::Yellow,
        };

        ElevatorInfra {
            carriage_shape,
            carriage_playground,
            marker: Marker::Block,
            tick_count: 0,
            dir_x: 0,
            dir_y: 0,
            building_wall: separator,
            // level_markers: floor_screen_markers,
            each_floor_height,
            // floor_coords,
            floor_as_rects: all_floors_represented_as_rects

        }
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

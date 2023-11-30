use std::rc::Rc;
use ratatui::{widgets::canvas::Rectangle, prelude::{Color, Marker, Rect}};

#[derive(Debug)]
pub struct CarriageParameters {
    pub carriage_shape: Rectangle,
    pub carriage_playground: Rect,
    // carriage_canvas_params: ((f64,f64),(f64,f64)), // X-bounds, Y-bounds,
    delta_x: f64,
    delta_y: f64,
    pub marker: Marker,
    tick_count: i32,
    dir_x: i16,
    dir_y: i16
}

impl  CarriageParameters {
    pub(crate) fn new(movement_area: Rect) -> Self {

        let marker = Marker::Braille;
        let carriage_playground = Rect {
            x:      movement_area.x,
            y:      movement_area.y,
            width:  movement_area.width,
            height: movement_area.height
        };
       /*  let carriage_canvas = Canvas::default()
            .block(
                Block::default()
                .borders(Borders::ALL)
                .title("Carriage")
            )
            .marker(marker)
            .x_bounds([1.0, carriage_playground.width as f64 - 1.0])
            .y_bounds([1.0, carriage_playground.height as f64 - 1.0])
            ; 
        */

        let carriage_shape = Rectangle {
            x: 1.0,
            y: 1.0,
            width: 10.0,
            height: 5.0,
            color: Color::Yellow,
        };

        CarriageParameters {
            carriage_shape,
            carriage_playground,
            delta_x: 1.0,
            delta_y: 1.0,
            marker: Marker::Braille,
            tick_count: 0,
            dir_x: 0,
            dir_y: 0
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

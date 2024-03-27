
use colored::Colorize;
use ratatui::{backend::Backend, layout::Alignment, style::{Color, Modifier, Style, Stylize}, symbols::Marker, text::{Line as TextLine, Span, Text}, widgets::{canvas::{Canvas, Line, Rectangle}, Block,Borders, Paragraph, Wrap}, Frame};
use tracing::info;

use crate::{elevator_infra::ElevatorVisualInfra, tui_layout::TuiLayout};
/* 
#[derive(Debug)]
pub struct CarriageShape {
    pub  bottom_left_x_offset_from_origin: f64,
    pub  bottom_left_y_offset_from_origin: f64,
    pub  width:  f64,
    pub  height: f64
}

impl CarriageShape {
    pub fn move_up(&mut self, displacement: f64) -> &mut Self {
        self.bottom_left_y_offset_from_origin += displacement;
        self
    }

    pub fn move_down(&mut self, displacement: f64) -> &mut Self {
        self.bottom_left_y_offset_from_origin -= displacement;
        self
    }

    pub fn move_to_ground(&mut self) -> &mut Self {
        self.bottom_left_x_offset_from_origin = 0.0;
        self
    }
} */

#[derive(Debug)]
pub struct DisplayManager {

    pub floors_origin_x: f64,
    pub floors_origin_y: f64

   /*  pub display_area_origin_x: f64,
    pub display_area_origin_y: f64,
    pub display_area_width   : u16,
    pub display_area_height  : u16,
    pub carriage             : CarriageShape, */
}

impl DisplayManager {
    pub fn new() -> DisplayManager { 
        DisplayManager{
            floors_origin_x: 0.0
,           floors_origin_y: 0.0
        }  // TODO: we don't need a newtype here! 
    }
    
     /*

    pub fn move_carriage_up(&mut self, displacement: (f64 /* x */, f64 /* y */ )) -> &mut DisplayManager {
        self.carriage.move_up(displacement.1);
        self
    }

    pub fn move_carriage_down(&mut self, displacement: (f64 /* x */, f64 /* y */ )) -> &mut DisplayManager {
        self.carriage.move_down(displacement.1);
        self
    } */

    fn create_state_description_paragraph(heading: &str,left: f64, top: f64, right: f64, bottom: f64) -> Paragraph<'static> {
        let text = Text::from(vec![
            TextLine::from(vec![
                Span::raw("First"),
                Span::styled("line",Style::new().green().italic()),
                ".".into(),
            ]),
            TextLine::from("Second line"),
            "Third line".into(),
            heading.to_owned().into(),
            format!("left {}, top {}, width {}, height {}", left,top,right,bottom).into()
        ]);
        let paragraph = Paragraph::new(text)
            .block(Block::new()
                .title("Elevator-Panel")
                .borders(Borders::ALL))
            .style(Style::new().white().on_black())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
    
        paragraph
    }
    
    fn display_inner_structure_as_paragraph(heading: &str,inner: &ElevatorVisualInfra, rectangles: &Vec<Rectangle>) -> Paragraph<'static> {
    
        let main_desc = vec![
            TextLine::from(vec![
                Span::raw("First"),
                Span::styled("line",Style::new().green().italic()),
                ".".into(),
            ]),
            TextLine::from("Second line"),
            "Third line".into(),
            heading.to_owned().into(),
            format!("carriage_playground.x {}, carriage_playground.y {}, carriage_playground.width {}, carriage_playground.height {}",inner.carriage_playground.x,inner.carriage_playground.y,inner.carriage_playground.width,inner.carriage_playground.height).into(),
            format!("rect {}, left {}, top {}, right {}, bottom {}",0,inner.floor_as_rects[0].left(), inner.floor_as_rects[0].top(), inner.floor_as_rects[0].right(), inner.floor_as_rects[0].bottom()).into(),
            format!("rect {}, left {}, top {}, right {}, bottom {}",1,inner.floor_as_rects[1].left(), inner.floor_as_rects[1].top(), inner.floor_as_rects[1].right(), inner.floor_as_rects[1].bottom()).into(),
            format!("rect {}, left {}, top {}, right {}, bottom {}",2,inner.floor_as_rects[2].left(), inner.floor_as_rects[2].top(), inner.floor_as_rects[2].right(), inner.floor_as_rects[2].bottom()).into(),
            format!("rect {}, left {}, top {}, right {}, bottom {}",5,inner.floor_as_rects[5].left(), inner.floor_as_rects[5].top(), inner.floor_as_rects[5].right(), inner.floor_as_rects[5].bottom()).into(),
            
            format!("wall, start_x {}, start_y {}, end_x {}, end_y {}",inner.building_wall.start_x,inner.
            building_wall.start_y,inner.building_wall.end_x,inner.building_wall.end_y).into(),

            format!("Rectangle {}, x {}, y {}, width {}, height {}", 0, rectangles[0].x,rectangles[0].y,rectangles[0].width,rectangles[0].height).into(),

            format!("Rectangle {}, x {}, y {}, width {}, height {}", 1, rectangles[1].x,rectangles[1].y,rectangles[1].width,rectangles[1].height).into(),

            format!("Rectangle {}, x {}, y {}, width {}, height {}", 4, rectangles[4].x,rectangles[4].y,rectangles[4].width,rectangles[4].height).into(),

            format!("Rectangle {}, x {}, y {}, width {}, height {}", 5, rectangles[5].x,rectangles[5].y,rectangles[5].width,rectangles[5].height).into(),

            
            format!("carriage, bottom_x {}, bottom_y {}", inner.carriage_box.bottom_left_x_offset_from_origin, inner.carriage_box.bottom_left_y_offset_from_origin).into()

    
        ];
    
        let text = Text::from(main_desc);
        let paragraph = Paragraph::new(text)
            .block(Block::new()
                .title("Elevator-Panel")
                .borders(Borders::ALL))
            .style(Style::new().white().on_black())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
    
        paragraph
    }
    
    
    /// Renders the user interface widgets.
    pub fn render_working (&mut self,infra: &ElevatorVisualInfra,layout: &TuiLayout, f: &mut Frame) {

        let elevator_monitor_layout = layout.input_window[0];
        let elevator_carriage_layout = layout.output_windows[1];
        let elevator_start_button = layout.button_windows[0];
        let elevator_stop_button = layout.button_windows[1];
        let elevator_current_floor = layout.button_windows[2];
        let elevator_next_floor = layout.button_windows[3];

        let label_currently_at = infra.current_floor.and_then(|v| {
                if v == 0 { Some("Ground floor".to_owned()) } 
                else {
                    let x = format!("Floor {}", v);
                    Some(format!("Floor {}", v)) 
                }
            })
            .or(Some(format!("Unknwon at the moment")))
            .unwrap();

        let label_next_stop = infra.dest_floor.and_then(|v| {
                if v == 0 { Some("Ground floor".to_owned()) } 
                else {
                    let x = format!("Floor {}", v);
                    Some(format!("Floor {}", v)) 
                }
            })
            .or(Some(format!("Unknwon at the moment")))
            .unwrap();

        f.render_widget(
            Block::new().borders(Borders::ALL).title("Elevator monitor, press 'q' to quit"),
                elevator_monitor_layout);

        f.render_widget(
            Paragraph::new("Press here to start.")
                .block(
                    Block::new()
                            .borders(Borders::ALL)
                            //.title("Press inside the box to start")
                            .bg(Color::Green)
                            .fg(Color::Black)
                ),
                elevator_start_button);

        f.render_widget(
            Paragraph::new("Press here to stop.")
                .block(
                    Block::new()
                    .borders(Borders::ALL)
                    //.title("Press inside the box to stop")
                    .bg(Color::Red)
                    .fg(Color::Black)
                ),
                elevator_stop_button);

        f.render_widget(
            Paragraph::new(label_currently_at)
                .style(Style::default().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(
                    Block::new()
                    .borders(Borders::ALL)
                    .title("Currently at floor")
                    .bg(Color::LightBlue)
                    .fg(Color::Black)
                ),
                elevator_current_floor);

        f.render_widget(
            Paragraph::new(label_next_stop)
                .style(Style::default().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::new()
                       .borders(Borders::ALL)
                       .title("Next stop at floor")
                       .bg(Color::LightMagenta)
                       .fg(Color::Black)
                    ),
                elevator_next_floor);            
       

       let output_chunks = layout.output_windows.clone();

       let floors_as_rectangles: Vec<Rectangle> = 
                    DisplayManager::translate_floor_coords_to_viewport_rectangles(infra, (0.0,0.0));

    
        let canvas = Canvas::default()
            .block(
                Block::default()
                .bg(Color::White)
                .borders(Borders::ALL)
                .title("Floors + Carriage")
                .style(Style::default().bg(Color::LightBlue).fg(Color::Gray))
            )
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                
                ctx.draw(&Line {
                            x1: (0.0 + (infra.carriage_playground.width as f64 / 2.0)),
                            y1:  0.0,
                            x2: (0.0 + (infra.carriage_playground.width as f64 / 2.0)),
                            y2: (0.0 + infra.carriage_playground.height as f64),
                            color:Color::Black
    
                });
    
    
                for each_floor_as_rectangle in &floors_as_rectangles{
                    ctx.draw(each_floor_as_rectangle);
                }
    
                if infra.should_show_carriage() {
                    ctx.draw(&self.bring_carriage_to_screen(infra));
                }
                    
            })
           
            .x_bounds([self.floors_origin_x, infra.carriage_playground.width as f64 ])
            .y_bounds([self.floors_origin_y, infra.carriage_playground.height as f64 ])
            
            ;


        f.render_widget(canvas, output_chunks[1]);
    
    }

   /*  pub fn create_rectangle_for_carriage(&self) -> Rectangle {
        Rectangle {
            x: self.display_area_origin_x + self.carriage.bottom_left_x_offset_from_origin,
            y: self.display_area_origin_y + self.carriage.bottom_left_y_offset_from_origin,
            width: self.display_area_width as f64 / 2.0,
            height: self.carriage.height,
            color: Color::LightGreen
        }
    } */

    pub fn bring_carriage_to_screen(&self, infra: &ElevatorVisualInfra) -> Rectangle {
        //info!("crriage bo {:?}", infra.carriage_box);
        Rectangle {
            x: self.floors_origin_x + infra.carriage_box.bottom_left_x_offset_from_origin,
            y: self.floors_origin_y + infra.carriage_box.bottom_left_y_offset_from_origin,
            width: infra.carriage_box.width,
            height: infra.carriage_box.height,
            color: Color::LightGreen
        }
    }
    
    pub fn translate_floor_coords_to_viewport_rectangles(
                infra: &ElevatorVisualInfra, origin: (f64,f64)) -> Vec<Rectangle> {            
    
        let f = infra.floor_as_rects.iter().enumerate()
                .rev()
                .map(|(index,next)| {
                    Rectangle {
                        x: origin.0,
                        y: (origin.1 + (next.height as f64 * index as f64)),
                        width: (infra.carriage_playground.width as f64 / 2.0),
                        height: infra.each_floor_height as f64,
                        color: if infra.floors_having_passengers[index] 
                                { Color::LightGreen }
                               else 
                                { Color::Gray  } 
                    }
                })
                .collect()
                ;                
        f                
    }
    
    fn display_floor_rectangles_as_paragraph(heading: &str,inner: &ElevatorVisualInfra,floor_rectangles: &Vec<Rectangle>) -> Paragraph<'static> {
    
        let mut t = Text::default();
    
    
        for (index,next) in floor_rectangles.iter().enumerate() {
            t.extend(Text::from(
                format!("index {}, rectangle.x {}, rectangle.y {}, rectangle.width {}, rectangle.height {}",
                index as u16, next.x, next.y, next.width, next.height
                )
            ));
        };
    
        let paragraph = Paragraph::new(t)
            .block(Block::new()
                .title("Elevator-Panel")
                .borders(Borders::ALL))
            .style(Style::new().white().on_black())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
    
        paragraph
    }
    
}



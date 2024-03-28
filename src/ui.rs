use std::collections::VecDeque;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::Line as TextLine,
    widgets::{
        canvas::{Canvas, Line, Rectangle},
        Block, Borders, Paragraph,
    },
    Frame,
};
use tracing::info;

use crate::{elevator_infra::ElevatorVisualInfra, tui_layout::TuiLayout};

#[derive(Debug)]
pub struct DisplayManager {
    pub floors_origin_x: f64,
    pub floors_origin_y: f64,
}

impl DisplayManager {
    pub fn new() -> DisplayManager {
        DisplayManager {
            floors_origin_x: 0.0,
            floors_origin_y: 0.0,
        } // TODO: we don't need a newtype here!
    }

    /// Renders the user interface widgets.
    pub fn render_working(
        &mut self,
        infra: &ElevatorVisualInfra,
        messages_for_ops: &VecDeque<String>,
        layout: &TuiLayout,
        f: &mut Frame,
    ) {
        let elevator_monitor_layout = layout.info_window[0];
        let _elevator_carriage_layout = layout.motion_window[1];
        let elevator_start_button = layout.button_windows[layout.start_button_index as usize];
        let elevator_stop_button = layout.button_windows[layout.stop_button_index as usize];
        let elevator_current_floor = layout.button_windows[layout.current_floor_index as usize];
        let elevator_next_floor = layout.button_windows[layout.next_stop_index as usize];

        let scroll_by = self.compute_scroll_extent(messages_for_ops, &elevator_monitor_layout);

        let label_currently_at = self.create_label_for_current_floor(infra);

        let label_next_stop = self.create_label_for_dest_floor(infra);

        self.render_elevator_monitor_window(
            messages_for_ops,
            elevator_monitor_layout,
            scroll_by,
            f,
        );

        self.render_start_button(elevator_start_button, f);

        self.render_stop_button(elevator_stop_button, f);

        self.render_currently_at_kiosk(label_currently_at, elevator_current_floor, f);

        self.render_next_stop_kiosk(label_next_stop, elevator_next_floor, f );

        let output_chunks = layout.motion_window.clone();

        let floors_as_rectangles: Vec<Rectangle> =
            DisplayManager::translate_floor_coords_to_viewport_rectangles(infra, (0.0, 0.0));

        let canvas = Canvas::default()
            .block(
                Block::default()
                    .bg(Color::White)
                    .borders(Borders::ALL)
                    .title("Floors + Carriage")
                    .style(Style::default().bg(Color::LightBlue).fg(Color::Gray)),
            )
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&Line {
                    x1: (0.0 + (infra.carriage_playground.width as f64 / 2.0)),
                    y1: 0.0,
                    x2: (0.0 + (infra.carriage_playground.width as f64 / 2.0)),
                    y2: (0.0 + infra.carriage_playground.height as f64),
                    color: Color::Black,
                });

                for each_floor_as_rectangle in &floors_as_rectangles {
                    ctx.draw(each_floor_as_rectangle);
                }

                if infra.should_show_carriage() {
                    ctx.draw(&self.bring_carriage_to_screen(infra));
                }
            })
            .x_bounds([self.floors_origin_x, infra.carriage_playground.width as f64])
            .y_bounds([
                self.floors_origin_y,
                infra.carriage_playground.height as f64,
            ]);

        //f.render_widget(canvas, output_chunks[1]);
        f.render_widget(canvas, _elevator_carriage_layout);
        
    }

    fn create_label_for_current_floor(&self, infra: &ElevatorVisualInfra) -> String {
        let label_currently_at = infra
            .current_floor
            .and_then(|v| {
                if v == 0 {
                    Some("Ground floor".to_owned())
                } else {
                    Some(format!("Floor {}", v))
                }
            })
            .or(Some(format!("Unknown at the moment")))
            .unwrap();

        label_currently_at
    }

    fn create_label_for_dest_floor(&self, infra: &ElevatorVisualInfra) -> String {
        let label_next_stop = infra
            .dest_floor
            .and_then(|v| {
                if v == 0 {
                    Some("Ground floor".to_owned())
                } else {
                    Some(format!("Floor {}", v))
                }
            })
            .or(Some(format!("Unknown at the moment")))
            .unwrap();

        label_next_stop
    }

    fn bring_carriage_to_screen(&self, infra: &ElevatorVisualInfra) -> Rectangle {
        //info!("crriage bo {:?}", infra.carriage_box);
        Rectangle {
            x: self.floors_origin_x + infra.carriage_box.bottom_left_x_offset_from_origin,
            y: self.floors_origin_y + infra.carriage_box.bottom_left_y_offset_from_origin,
            width: infra.carriage_box.width,
            height: infra.carriage_box.height,
            color: Color::LightGreen,
        }
    }

    fn translate_floor_coords_to_viewport_rectangles(
        infra: &ElevatorVisualInfra,
        origin: (f64, f64),
    ) -> Vec<Rectangle> {
        let f = infra
            .floor_as_rects
            .iter()
            .enumerate()
            .rev()
            .map(|(index, next)| Rectangle {
                x: origin.0,
                y: (origin.1 + (next.height as f64 * index as f64)),
                width: (infra.carriage_playground.width as f64 / 2.0),
                height: infra.each_floor_height as f64,
                color: if infra.floors_having_passengers[index] {
                    Color::LightGreen
                } else {
                    Color::Gray
                },
            })
            .collect();
        f
    }

    fn compute_scroll_extent(
        &self,
        messages_for_ops: &VecDeque<String>,
        displayable_area: &Rect,
    ) -> u16 {
        let content_lines = messages_for_ops.len();
        let displayable_area_lines = displayable_area.height as usize;

        if content_lines <= displayable_area_lines {
            0
        } else {
            (content_lines - displayable_area_lines) as u16
        }
    }

    fn render_elevator_monitor_window(
        &mut self,
        messages_for_ops: &VecDeque<String>,
        elevator_monitor_layout: Rect,
        scroll_by: u16,
        f: &mut Frame,
    ) -> () {
        f.render_widget(
            Paragraph::new(
                messages_for_ops
                    .clone()
                    .drain(0..)
                    .map(|n| TextLine::from(n))
                    .collect::<Vec<TextLine>>(),
            )
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Elevator monitor, press 'q' to quit"),
            )
            .scroll((scroll_by, 0)),
            elevator_monitor_layout,
        );
    }

    fn render_start_button(&self, elevator_start_button: Rect, f: &mut Frame) -> () {
        f.render_widget(
            Paragraph::new("Press here to start.").block(
                Block::new()
                    .borders(Borders::ALL)
                    .bg(Color::Green)
                    .fg(Color::Black),
            ),
            elevator_start_button,
        );
    }

    fn render_stop_button(&self, elevator_stop_button: Rect, f: &mut Frame) -> () {
        f.render_widget(
            Paragraph::new("Press here to stop.").block(
                Block::new()
                    .borders(Borders::ALL)
                    .bg(Color::Red)
                    .fg(Color::Black),
            ),
            elevator_stop_button,
        );
    }

    fn render_currently_at_kiosk(
        &self,
        label_currently_at: String,
        elevator_current_floor: Rect,
        f: &mut Frame,
    ) -> () {
        f.render_widget(
            Paragraph::new(label_currently_at)
                .style(Style::default().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .title("Currently at floor")
                        .bg(Color::LightBlue)
                        .fg(Color::Black),
                ),
            elevator_current_floor,
        );
    }

    fn render_next_stop_kiosk(
        &self,
        label_next_stop: String,
        elevator_next_floor: Rect,
        f: &mut Frame,
    ) -> () {
        f.render_widget(
            Paragraph::new(label_next_stop)
                .style(Style::default().add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .title("Next stop at floor")
                        .bg(Color::LightMagenta)
                        .fg(Color::Black),
                ),
            elevator_next_floor,
        );
    }
}

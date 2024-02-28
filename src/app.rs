use std::error;
use crate::{machinery::CarriageActor, elevator_infra::{ElevatorInfra}, async_event::AppOwnEvent};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;
use crossterm::event::MouseEventKind::Down;
use crossterm::event::MouseButton::Left;

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
                    match m.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            if let Some(floor) = 
                                self.inner_display_setup
                                .is_passenger_at_reachable_floor(
                                    Position{x: m.column, y: m.row}
                                ) {

                                self.inner_display_setup.on_passenger_summoning_to_floor(floor);
                            };
                        
                        }, 
                        _ => {} // ignore other mouse events
                    
                    };
                },
                _ => {}  //ignore other App events
            };
        }
    }

    pub fn save_to_file(&self) -> () {

        /* for p in self.inner_display_setup.floor_as_rects.iter().enumerate() {
            println!("Rect index {}", p.0);
            for j in p.1.positions() {
                println!("{:?}", j);
            }
           
        } */
        

        for n in &self.mouse_at {
            /* let maybe_floor = self.inner_display_setup.indicate_floor_chosen((n.column,n.row)); */

            match n {
                MouseEvent { kind: Down(_), column: c, row: r, modifiers: _ } => {
                    for next_floor in self.inner_display_setup.floor_as_rects.iter().enumerate() {
                        let is_in = next_floor.1.contains(Position{x: *c, y: *r });
                        println!(" Mouse-event kind {:?}, at (Row{} : Column{}), Rect [{}] (left {}, top {}, right {}, bottom {}, contains? {}", 
                                    n.kind, 
                                    n.row, 
                                    n.column,
                                    next_floor.0,
                                    next_floor.1.left(),
                                    next_floor.1.top(),
                                    next_floor.1.right(),
                                    next_floor.1.bottom(),
                                    if is_in { "Yes".to_owned() } else { "No".to_owned() }
                            );
                        }
                        /* println!(" Mouse-event kind {:?}, at (Row{} : Column{})", 
                                    n.kind, 
                                    n.row, 
                                    n.column,
                            ); */

                    },
                _ => {}
            }
 
        }


        /*     println!(" Mouse-event kind {:?}, at Row{}:Column{}, floor {}", 
                            n.kind, 
                            n.row, 
                            n.column,
                            maybe_floor.map_or(10, |x| x)
                    ); */
    }
 }
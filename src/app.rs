use std::{error, io::Stdout};
use crate::{async_event::AppOwnEvent, elevator_infra::ElevatorInfra, tui::Tui, tui_layout::TuiLayout, ui::DisplayManager};
use crossterm::event::{self, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{backend::{Backend, CrosstermBackend}, layout::Position, Terminal};
use crossterm::event::MouseEventKind::Down;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::info;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App <B: Backend> {
    pub inner_infra: ElevatorInfra,
    to_quit: bool,
    pub tick_count: i32,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub last_tick_key_events: Vec<KeyEvent>,
    pub mouse_at: Vec<MouseEvent>,
    pub tui_wrapper: Tui<B>,
    pub event_rx: UnboundedReceiver<AppOwnEvent>,
    pub event_tx: UnboundedSender<AppOwnEvent>,
    // pub elevator_control: ElevatorControllerActor
    // pun eleva 

}

impl<B: Backend> App <B> {

    pub fn new (carriage_movement_area: ElevatorInfra,tick_rate: f64, frame_rate: f64, terminal: Terminal<B>, tui_layout: TuiLayout, ui: DisplayManager) -> Self { 
        let (event_tx, event_rx) = mpsc::unbounded_channel();
      
        let tui = Tui::new(terminal,tui_layout,ui, event_tx.clone());

        Self {
            inner_infra: carriage_movement_area,
            to_quit: false,
            tick_count: 0i32,
            tick_rate,
            frame_rate,
            last_tick_key_events: Vec::new(),
            mouse_at: Vec::new(),
            tui_wrapper: tui,
            event_rx,
            event_tx
        } 
    }

    pub fn quit(&mut self) -> () {
        self.to_quit = true;
        () 
    }


    pub fn should_quit_app(&self) -> bool {
        self.to_quit
    }

    pub fn init(&mut self) -> AppResult<()> {
        self.tui_wrapper.init()
    }

    pub fn start(&mut self) ->  AppResult<()> {
        self.tui_wrapper.start()
    }

    pub async fn run(&mut self) -> AppResult<()> {
        
        let floor_locations = self.inner_infra.get_carriage_displacement_map_per_floor((0,0));
        let current_left_top_y_init = floor_locations[0].0; // Ground floor
        let mut current_left_top_y = current_left_top_y_init; 
        
        loop {
            
            // Treat events from Tui and Elevator, as appropriate.
            match self.event_rx.recv().await {

                Some(AppOwnEvent::Init) => info!("app received init!"),

                Some(AppOwnEvent::Tick) => {
    
                    if let Some(floor) = self.inner_infra.is_any_passenger_waiting() {
                        info!("First log message here");
                        if current_left_top_y < floor_locations[floor as usize].0 {
                            current_left_top_y += 1.0;
                            self.inner_infra.on_tick((0,1));
                            self.tui_wrapper.ui.move_carriage_up((0.0,1.0));
                        }
                    }
                    else {
                        info!("Tick received, no passenger waiting");
                    }
                   
                },
                Some(AppOwnEvent::Render) => {
                    self.tui_wrapper.draw(&self.inner_infra)?;
                },
                Some(AppOwnEvent::Key(key_event)) => {
                    self.update_app(AppOwnEvent::Key(key_event))
                    
                },
                e@ Some(AppOwnEvent::Mouse(m)) => {
                    self.update_app(e.unwrap())
                },
                None => {},
                _ => {}
                
            }
    
            if self.should_quit_app() {
                self.tui_wrapper.exit()?;
                break Ok(());
            }
                
        }
    }

    pub fn update_app(&mut self, app_event: AppOwnEvent) -> () {

        if let AppOwnEvent::Key(key) = app_event {
            match key.code {
              KeyCode::Char('q') => { 
                // TODO: Inform the actors
                // TODO: Bring Tui back to cooked mode
                self.quit() },
                 
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
                            if let Some(floor_no) = 
                                self.inner_infra
                                .is_passenger_waiting_at_reachable_floor(
                                    Position{x: m.column, y: m.row}
                                ) {

                                self.inner_infra.on_passenger_summoning(floor_no);
                                // TODO: Send a ElevatorVocabulary::Moveto message to ControllerActor
                            };

                            // TODO: Else, if Mouse-click indicates that Elevator should start
                            // Else, if Mouse-click indicates that Elevator should stop
                        
                        }, 
                        _ => {} // ignore other mouse events
                    
                    };
                },
                _ => {}  //ignore other App events
            };
        }
    }

    pub fn save_to_file(&self) -> () {

        for n in &self.mouse_at {

            match n {
                MouseEvent { kind: Down(_), column: c, row: r, modifiers: _ } => {
                    for next_floor in self.inner_infra.floor_as_rects.iter().enumerate() {
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

                    },
                _ => {}
            }
 
        }

    }
 }
use crate::app_own_event::AppOwnEvent;
use crate::elevator_installation::elevator_service::PassengerLiftActor;
use crate::{
    
    conversation::vocabulary::{ElevatorVocabulary, PulleyVocabulary},
    elevator_infra::{ElevatorVisualInfra, MX_FLOORS},
    elevator_installation::pulley_machinery::PulleyActor,
    tui::Tui,
    tui_layout::TuiLayout,
    ui::DisplayManager,
};

use crossterm::event::{KeyCode, MouseButton, MouseEventKind};
use ractor::{Actor, ActorRef};
use ratatui::{
    backend::Backend,
    layout::Position,
    Terminal,
};
use std::{collections::VecDeque, error, time::Duration};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::info;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App<B: Backend> {
    pub inner_infra: ElevatorVisualInfra,
    to_quit: bool,
    pub tick_count: i32,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub tui_wrapper: Tui<B>,
    pub app_own_event_rx: UnboundedReceiver<AppOwnEvent>,
    pub app_own_event_tx: UnboundedSender<AppOwnEvent>,
    pub elev_event_tx: UnboundedSender<ElevatorVocabulary>,
    pub elev_event_rx: UnboundedReceiver<ElevatorVocabulary>,
    passenger_lift: (ActorRef<ElevatorVocabulary>, JoinHandle<()>),
    pulley_machinery: (ActorRef<PulleyVocabulary>, JoinHandle<()>),
    messages_for_ops: VecDeque<String>,
}

impl<B: Backend> App<B> {
    pub async fn new(
        carriage_movement_area: ElevatorVisualInfra,
        tick_rate: f64,
        frame_rate: f64,
        terminal: Terminal<B>,
        tui_layout: TuiLayout,
        ui: DisplayManager,
    ) -> Self {
        let (app_own_event_tx, app_own_event_rx) = mpsc::unbounded_channel();

        let tui = Tui::new(terminal, tui_layout, ui, app_own_event_tx.clone());

        let floor_setting = carriage_movement_area.get_carriage_displacement_map_per_floor((0, 0));

        let (pulley_ref, pulley_handle) = Actor::spawn(
            Some(String::from("Pulley_actor")),
            PulleyActor,
            floor_setting,
        )
        .await
        .expect("Failed to create Pulley actor");

        let (elev_event_tx, elev_event_rx) = mpsc::unbounded_channel();

        let (elev_ref, elev_handle) = Actor::spawn(
            Some(String::from("Elevator-Actor")),
            PassengerLiftActor,
            (MX_FLOORS, Some(elev_event_tx.clone()), pulley_ref.clone()),
        )
        .await
        .expect("Failed to start actor");

        Self {
            inner_infra: carriage_movement_area,
            to_quit: false,
            tick_count: 0i32,
            tick_rate,
            frame_rate,
            tui_wrapper: tui,
            app_own_event_rx,
            app_own_event_tx,
            elev_event_tx,
            elev_event_rx,
            passenger_lift: (elev_ref, elev_handle),
            pulley_machinery: (pulley_ref, pulley_handle),
            messages_for_ops: VecDeque::with_capacity(1024),
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

    pub fn start(&mut self) -> AppResult<()> {
        self.tui_wrapper.start()
    }

    pub async fn run(&mut self) -> AppResult<()> {
        loop {
            tokio::select! {
                from_elevator = self.elev_event_rx.recv() => {
                    match from_elevator {
                       Some(ElevatorVocabulary::MoveToGroundFloor) => {
                        self.messages_for_ops.push_back(String::from("Elevator is moving to ground-floor."));
                        self.inner_infra.set_carriage_ready();
                       },
                       Some(ElevatorVocabulary::MovingTo(f)) => {
                        self.messages_for_ops.push_back(format!("Elevator is moving to floor({})",f));
                        self.inner_infra.set_next_destination(f as u16);
                       }
                       Some(ElevatorVocabulary::CurrentCarriagePosn((x_posn,y_posn))) =>  {
                        self.inner_infra.on_carriage_moving_to((x_posn,y_posn));
                       },
                       Some(ElevatorVocabulary::OpenTheDoor(f)) => {
                        self.messages_for_ops.push_back(format!("Elevator has reached floor({}), door is open.",f));
                        self.inner_infra.on_reaching_destination();

                        // We are simulating the action of opening, waiting and closing the carriage-door.
                        let app_event_channel_passed = self.app_own_event_tx.clone();
                        let _ = tokio::spawn (async move {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            app_event_channel_passed.send(AppOwnEvent::AllPassengersAlighted(f)).unwrap();
                        });

                       }
                       Some(ElevatorVocabulary::ElevatorOutOfService) => {
                        self.messages_for_ops.push_back(String::from("Elevator is not operating anymore!"));
                        self.inner_infra.unset_carriage();
                        self.messages_for_ops.push_back(String::from("Quitting application in 3 seconds."));
                        // We are simulating the action of notifying that we are exiting.
                        let app_event_channel_passed = self.app_own_event_tx.clone();
                        let _ = tokio::spawn (async move {
                            tokio::time::sleep(Duration::from_secs(3)).await;
                            app_event_channel_passed.send(AppOwnEvent::Exit).unwrap();
                        });
                       },
                       Some(ElevatorVocabulary::Stop(0)) => {},
                       Some(_) => {},
                       None => { todo!(); }

                }},
                app_own_event = self.app_own_event_rx.recv() => {
                    self.handle_app_own_event(app_own_event).unwrap();
                }
            };

            if self.should_quit_app() {
                self.tui_wrapper.exit()?;
                break Ok(());
            }
        }
    }

    fn handle_app_own_event(&mut self, e: Option<AppOwnEvent>) -> AppResult<()> {
        match e {
            Some(AppOwnEvent::Init) => info!("app received init!"),

            Some(AppOwnEvent::Tick) => {}

            Some(AppOwnEvent::Exit) => self.quit(),

            Some(AppOwnEvent::AllPassengersAlighted(at_floor)) => {
                self.messages_for_ops
                    .push_back(format!("Passengers have alighted at floor ({}). Door is closed.",at_floor));
                self.inner_infra
                    .mark_floor_on_reaching_destination(at_floor as u16);
                self.passenger_lift
                    .0
                    .send_message(ElevatorVocabulary::DoorClosed(at_floor))
                    .unwrap();
            }

            Some(AppOwnEvent::Render) => 
                self.tui_wrapper
                    .draw(&self.inner_infra, &self.messages_for_ops)?,
            Some(AppOwnEvent::Key(key_event)) =>
                self.on_inputs_from_users(AppOwnEvent::Key(key_event)),
            e @ Some(AppOwnEvent::Mouse(_)) => 
                self.on_inputs_from_users(e.unwrap()),
            None => {}
            _ => {}
        }

        Ok(())
    }

    pub fn on_inputs_from_users(&mut self, app_event: AppOwnEvent) -> () {
        match app_event {
            AppOwnEvent::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    info!("'q' pressed, elevator app is exiting.");
                    self.messages_for_ops
                        .push_back(String::from("Operator has pressed 'q'. Will exit."));
                    self.quit()
                }

                _ => {}
            },

            AppOwnEvent::Mouse(m) => {
                match m.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if let Some(floor_no) = self
                            .inner_infra
                            .is_passenger_waiting_at_reachable_floor(Position {
                                x: m.column,
                                y: m.row,
                            })
                        {
                            self.messages_for_ops
                                .push_back(format!("Passenger is waiting at {}!", floor_no));
                            self.inner_infra.serve_passenger_at(floor_no);
                            self.passenger_lift
                                .0
                                .send_message(
                                    ElevatorVocabulary::MoveToFloor(floor_no as u8), // TODO: do we need u16?
                                )
                                .unwrap();
                        } else if self.has_operator_pressed_start_button(Position {
                            x: m.column,
                            y: m.row,
                        }) {
                            info!("Elevator is starting!");
                            self.messages_for_ops
                                .push_back(String::from("Elevaror is starting!"));
                            self.passenger_lift
                                .0
                                .send_message(ElevatorVocabulary::PowerOn)
                                .unwrap();
                        } else if self.has_operator_pressed_button_stop_button(Position {
                            x: m.column,
                            y: m.row,
                        }) {
                            info!("Elevator is stopping!");
                            self.messages_for_ops
                                .push_back(String::from("Elevator is stopping!"));
                            self.passenger_lift
                                .0
                                .send_message(ElevatorVocabulary::PowerOff)
                                .unwrap();
                        } else {
                        }
                    }
                    _ => {} // ignore other mouse events
                };
            }
            _ => {} //ignore other App events
        };
    }


    fn has_operator_pressed_start_button(&self, p: Position) -> bool {
        self.tui_wrapper.layout.button_windows[0].contains(p)
    }

    fn has_operator_pressed_button_stop_button(&self, p: Position) -> bool {
        self.tui_wrapper.layout.button_windows[1].contains(p)
    }
}

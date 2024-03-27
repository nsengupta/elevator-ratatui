use crate::elevator_installation::carriage_internal::CarriageData;
use crate::conversation::vocabulary::ElevatorVocabulary;
use crate::conversation::vocabulary::PulleyVocabulary;
use crate::elevator_installation::elevator_operations::ElevatorFSMInputs::*;
use crate::elevator_installation::elevator_operations::ElevatorFSMOutputs::*;
use crate::elevator_installation::elevator_operations::ElevatorFSMStates::*;

use ractor::ActorRef;
use rust_fsm::*;
use tokio::sync::mpsc::UnboundedSender;

const MX_FLOORS: u32 = 7;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NextDestTodo {
    EmergencyAtGroundFloorAlready,
    EmergencyGotoGroundFloorNow,
    AllFineGotoNextPassenger(u8)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElevatorFSMStates {
    Moving,
    ReadyForService,
    UnavailableForService,
    ShuttingDown,
    NonOperational,
    DoorIsOpen,
    PoweredOn,
    PoweredOff
}

#[derive(Debug)]
pub enum ElevatorFSMInputs {
    SwitchOn,
    SwitchOff,
    MoveTo(u8),
    DoorClosed,
    Stop,
}

#[derive(Debug, PartialEq)]
pub enum ElevatorFSMOutputs {
    SettleAtGroundFloor,
    NextDest(u8),
    Enqueue(u8),
    Reached,
    CheckNextDest,
    PrepareForEmergencyStop,
    ExitPassengers,
}

pub struct ElevatorController<T: StateMachineImpl> {
    pub carriage_data: CarriageData,
    pub op_informant_channel: Option<UnboundedSender<ElevatorVocabulary>>,
    carriage_state_machine: StateMachine<T>,
    pub pulley_actor: ActorRef<PulleyVocabulary>,
}

impl ElevatorController<ElevatorStateMachine> {
    pub fn new(
        carriage_data: CarriageData,
        op_informant_channel: Option<UnboundedSender<ElevatorVocabulary>>,
        pulley_actor: ActorRef<PulleyVocabulary>,
    ) -> Self {
        Self {
            carriage_data: carriage_data,
            carriage_state_machine: StateMachine::new(),
            op_informant_channel,
            pulley_actor: pulley_actor,
        }
    }

    pub fn current_state(&mut self) -> ElevatorFSMStates {
        self.carriage_state_machine.state().clone()
    }

    pub fn already_at_floor(&self, floor_index: u8) -> bool {
        self.carriage_data.already_at_floor(floor_index)
    }

    pub fn set_next_destination(&mut self, dest_floor: u8) -> u8 {
        self.carriage_data.set_next_destination(dest_floor)
    }

    pub fn add_to_destinations_queue(&mut self, floor_id: u8) -> () {
        self.carriage_data.enqueue_next_destination(floor_id)
    }

    pub fn on_arrival(&mut self) -> u8 {
        self.carriage_data.on_arrival()
    }

    pub fn 
    on_checking_next_dest(&mut self) -> Option<NextDestTodo> {

        // If an emergency-stop (PowerOff) has been received already, then the carriage must
        //      1) Halt if it is already at groundfloor (floor 0)
        //      2) Move to the groundfloor (floor 0) if it is at any other floor
        //         ignoring all the next destinations requested for, by passengers
        //         at various floors. Safety is paramount!
        // Otherwise, Move to the next destination floor

        if self.carriage_data.is_emergency_op_requested() {
            if self.carriage_data.already_at_floor(0) {
                Some(NextDestTodo::EmergencyAtGroundFloorAlready) // TODO: Use enum here
            } else {
                self.set_next_destination(0);
                Some(NextDestTodo::EmergencyGotoGroundFloorNow)
            }
        } else if self.carriage_data.any_destination_in_queue() {
            let next_destination = self.carriage_data.dequeue_next_destination();
            self.set_next_destination(next_destination.unwrap());
            Some(NextDestTodo::AllFineGotoNextPassenger(next_destination.unwrap()))
        } else {
            None
        }
    }

    pub fn on_emergency(&mut self) -> () {
        self.carriage_data.prepare_for_emergency();
    }

    pub fn run_machine(
        &mut self,
        input: &ElevatorFSMInputs,
    ) -> (ElevatorFSMStates, Option<ElevatorFSMOutputs>) {
        if let Ok(maybe_output) = self.carriage_state_machine.consume(input) {
            (self.current_state(), maybe_output)
        } else {
            (self.current_state(), None)
        }
    }
}

pub struct ElevatorStateMachine {}
impl StateMachineImpl for ElevatorStateMachine {
    type Input = ElevatorFSMInputs;
    type State = ElevatorFSMStates;
    type Output = ElevatorFSMOutputs;
    const INITIAL_STATE: Self::State = PoweredOff;

    fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        match (state, input) {
            (PoweredOff, SwitchOn) => Some(ReadyForService),
            (ReadyForService, SwitchOff) => Some(UnavailableForService),
            (ReadyForService, MoveTo(_)) => Some(Moving),
            (UnavailableForService, MoveTo(_)) => Some(ShuttingDown),
            (Moving, MoveTo(_)) => Some(Moving),
            (Moving, Stop) => Some(DoorIsOpen),
            (Moving, SwitchOff) => Some(Moving),
            (ShuttingDown, MoveTo(0)) => Some(ShuttingDown),
            (DoorIsOpen, MoveTo(_)) => Some(DoorIsOpen),
            (DoorIsOpen, SwitchOff) => Some(DoorIsOpen),
            (DoorIsOpen, DoorClosed) => Some(ReadyForService),
            (ShuttingDown, Stop) => Some(NonOperational),
            (NonOperational, DoorClosed) => Some(PoweredOff),

            _ => None,
        }
    }

    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
        match (state, input) {
            (PoweredOff, SwitchOn) => Some(SettleAtGroundFloor),
            (ReadyForService, SwitchOff) => Some(SettleAtGroundFloor),
            (ReadyForService, MoveTo(floor)) => Some(NextDest(*floor)),
            (UnavailableForService, MoveTo(_)) => Some(SettleAtGroundFloor),
            (Moving, MoveTo(floor)) => Some(Enqueue(*floor)),
            (Moving, SwitchOff) => Some(PrepareForEmergencyStop),
            (DoorIsOpen, MoveTo(floor)) => Some(Enqueue(*floor)),
            (DoorIsOpen, SwitchOff) => Some(PrepareForEmergencyStop),
            (DoorIsOpen, DoorClosed) => Some(CheckNextDest),
            (Moving, Stop) => Some(Reached),
            (ShuttingDown, Stop) => Some(ExitPassengers),

            _ => None,
        }
    }
}

mod tests {
    use ractor::{Actor, ActorStatus};
    use tokio::sync::mpsc;

    use crate::elevator_installation::pulley_machinery::PulleyActor;

    use super::*;

    #[test]
    fn carriage_is_powered_when_switched_on() {
        let mut elevator_fsm: StateMachine<ElevatorStateMachine> = StateMachine::new();
        let _ = elevator_fsm.consume(&ElevatorFSMInputs::SwitchOn);
        assert_eq!(elevator_fsm.state(), &ElevatorFSMStates::ReadyForService);
    }

    #[tokio::test]
    async fn carriage_is_at_ground_floor_when_started() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let carriage_data = CarriageData::new(8);
        let floor_setting = vec![(0.0, 5.0), (5.0, 10.0), (10.0, 15.0), (15.0, 20.0)];
        let (pulley_ref, pulley_handle) = Actor::spawn(
            Some(String::from("Test_pulley_actor")),
            PulleyActor,
            floor_setting,
        )
        .await
        .expect("Failed to create Pulley actor");

        let mut carriage =
            ElevatorController::new(carriage_data, Some(tx.clone()), pulley_ref.clone());
        let maybe_output = carriage.run_machine(&ElevatorFSMInputs::SwitchOn);

        assert_eq!(
            carriage.current_state(),
            ElevatorFSMStates::ReadyForService,
            "Carriage State = READY"
        );

        assert_eq!(
            maybe_output.1.unwrap(),
            SettleAtGroundFloor,
            "FSM Output = SettleAtGroundFloor"
        );

        assert_eq!(
            carriage.already_at_floor(0),
            true,
            "Carriage current floor = 0"
        );

        pulley_ref.send_message(PulleyVocabulary::PowerOff).unwrap();

        println!("pulley status  {:?}", pulley_ref.get_status());

        pulley_handle.await.unwrap();

        loop {
            let st = pulley_ref.get_status();
            println!("Pulley status {:?}", st);
            if st == ActorStatus::Stopped {
                break;
            }
        }

        println!("Elevtor Controller, pulley actor is powered off");
        rx.close();
        drop(tx);
    }

    #[tokio::test]
    async fn when_passenger_dest_is_notified_to_stationery_carriage_then_it_begins_to_move() {
        let carriage_data = CarriageData::new(8);
        let floor_setting = vec![(0.0, 5.0), (5.0, 10.0), (10.0, 15.0), (15.0, 20.0)];
        let (pulley_ref, _) = Actor::spawn(
            Some(String::from("Test_pulley_actor-1")),
            PulleyActor,
            floor_setting,
        )
        .await
        .expect("Failed to create Pulley actor-1");

        let mut carriage = ElevatorController::new(carriage_data, None, pulley_ref);
        let _ = carriage.run_machine(&ElevatorFSMInputs::SwitchOn);
        let _ = carriage.run_machine(&MoveTo(2));
        assert_eq!(carriage.current_state(), ElevatorFSMStates::Moving);
    }

    #[tokio::test]
    async fn carriage_keeps_moving_when_new_passenger_destination_arrives() {
        let carriage_data = CarriageData::new(8);
        let floor_setting = vec![(0.0, 5.0), (5.0, 10.0), (10.0, 15.0), (15.0, 20.0)];
        let (pulley_ref, _) = Actor::spawn(
            Some(String::from("Test_pulley_actor-2")),
            PulleyActor,
            floor_setting,
        )
        .await
        .expect("Failed to create Pulley actor-2");
        let mut carriage = ElevatorController::new(carriage_data, None, pulley_ref);
        let _ = carriage.run_machine(&ElevatorFSMInputs::SwitchOn);
        let _ = carriage.run_machine(&ElevatorFSMInputs::MoveTo(2));

        // Another passenger input arrives, while the carriage is *still* Moving
        let maybe_output = carriage.run_machine(&ElevatorFSMInputs::MoveTo(5));
        assert_eq!(maybe_output.0, ElevatorFSMStates::Moving);
        assert_eq!(maybe_output.1, Some(ElevatorFSMOutputs::Enqueue(5)));

    }
}

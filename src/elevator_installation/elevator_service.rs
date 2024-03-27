use async_trait::async_trait;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use crate::elevator_installation::carriage_internal::CarriageData;
use crate::conversation::vocabulary::{ElevatorVocabulary, PulleyVocabulary};
use crate::conversation::vocabulary::ElevatorVocabulary::*;
use crate::elevator_installation::elevator_operations::{ElevatorController, ElevatorFSMInputs, ElevatorFSMOutputs, ElevatorFSMStates, ElevatorStateMachine, NextDestTodo};



#[cfg(feature = "cluster")]
impl ractor::Message for ElevatorVocabulary {}

pub struct PassengerLiftActor;
#[async_trait]
impl Actor for PassengerLiftActor {
    type Msg = ElevatorVocabulary;
    type State = ElevatorController<ElevatorStateMachine>;
    type Arguments = (u16,Option<UnboundedSender<ElevatorVocabulary>>,ActorRef<PulleyVocabulary>);

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, args: Self::Arguments) -> 
        Result<Self::State, ActorProcessingErr> {
            let carriage_data = CarriageData::new(args.0);
            Ok(ElevatorController::new(carriage_data,args.1,args.2))
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        elevator_control: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {

        match message {
            ElevatorVocabulary::PowerOn => {
                let _mc_run_outcome = elevator_control
                        .run_machine(&ElevatorFSMInputs::SwitchOn);
                info!("Event (PowerOn), was on floor: ({}), Transition(State: ({:?}), Outcome: ({:?}))", 
                        elevator_control.carriage_data.where_is(),
                        _mc_run_outcome.0, 
                        _mc_run_outcome.1
                );    
                
                match _mc_run_outcome {
                    (_, Some(ElevatorFSMOutputs::SettleAtGroundFloor)) => {
                        elevator_control
                        .pulley_actor
                        .send_message(PulleyVocabulary::PowerOn(myself.clone()))
                        .unwrap();
                        
                        elevator_control.set_next_destination(0);    
                        
                        elevator_control
                        .op_informant_channel.as_ref().map(|channel| {
                            channel.send(MoveToGroundFloor).unwrap();
                        });
                    },
                    (_, None)   => { 
                        info!("State {:?}, Transition outcome {}", 
                            _mc_run_outcome.0, 
                            "is NOP"
                        )},
                    (_, _) => {}
                }

            },
            ElevatorVocabulary::PowerOff => {
                let _mc_run_outcome = 
                        elevator_control.run_machine(&ElevatorFSMInputs::SwitchOff);
                info!("Event (PowerOff), was on floor:: ({}), Transition(State: ({:?}), Outcome: ({:?}))", 
                        elevator_control.carriage_data.where_is(),
                        _mc_run_outcome.0, 
                        _mc_run_outcome.1
                );
                match _mc_run_outcome {
                    (_, Some(ElevatorFSMOutputs::PrepareForEmergencyStop)) => {
                        elevator_control.on_emergency();
                    },
                    (_, Some(ElevatorFSMOutputs::SettleAtGroundFloor)) => {
                        elevator_control
                        .op_informant_channel.as_ref().map(|channel| {
                            channel.send(MovingTo(0)).unwrap_or_else(|e| {
                                info!("Receiver stopped, {:?}",e.0);
                            });
                        });

                        elevator_control.set_next_destination(0);

                        myself.send_message(ElevatorVocabulary::MoveToFloor(0)).unwrap();
                    },
                    (_, None)   => { 
                        info!("State {:?}, Transition outcome {}", 
                            _mc_run_outcome.0, 
                            "is NOP"
                        )},
                    (_, _) => {}
                }
            },
            ElevatorVocabulary::MoveToFloor(dest_floor) => {
                let _mc_run_outcome = elevator_control.run_machine(&ElevatorFSMInputs::MoveTo(dest_floor));
                info!("Event (MoveToFloor({})), was on floor: ({}), Transition(State: ({:?}), Outcome: ({:?}))", 
                    dest_floor,
                    elevator_control.carriage_data.where_is(),
                    _mc_run_outcome.0, 
                    _mc_run_outcome.1
                )
                ;
                match _mc_run_outcome {
                    (ElevatorFSMStates::ShuttingDown, Some(ElevatorFSMOutputs::SettleAtGroundFloor)) => {

                        if elevator_control.already_at_floor(0) {
                            elevator_control.
                            op_informant_channel.as_ref().map(|channel|{
                                channel.send(Stop(0)).unwrap();
                            });

                            myself.send_message(Stop(0)).unwrap();
                        }
                        else {
                            elevator_control.set_next_destination(0);
                            elevator_control
                            .op_informant_channel.as_ref().map(|channel| {
                                channel.send(MovingTo(0)).unwrap();
                            });
                            elevator_control
                            .pulley_actor
                            .send_message(PulleyVocabulary::MoveToFloor(0))
                            .unwrap();
                        }
                        
                    },
                    (_, Some(ElevatorFSMOutputs::NextDest(dest_floor))) => {
                        elevator_control.set_next_destination(dest_floor);
                        elevator_control
                        .op_informant_channel.as_ref().map(|channel| {
                            channel.send(MovingTo(dest_floor)).unwrap();
                        });
                        elevator_control
                        .pulley_actor
                        .send_message(PulleyVocabulary::MoveToFloor(dest_floor))
                        .unwrap();
                    },
                    (_, Some(ElevatorFSMOutputs::Enqueue(dest_floor))) => {
                        elevator_control.add_to_destinations_queue(dest_floor);
                    },
                    (_, None) => {},
                    (_, _)    => {} 
                };
            },

            ElevatorVocabulary::CurrentCarriagePosn((x,y)) => {
                info!("Event (CurrentCarriagePosn({},{})), Current State ({:?})",
                            x,
                            y,
                            elevator_control.current_state()
                        );
                elevator_control
                .op_informant_channel.as_ref().map(|channel| {
                    channel.send(ElevatorVocabulary::CurrentCarriagePosn((x,y)))
                    .unwrap();
                });
            },

            ElevatorVocabulary::Stop(at_floor) => {
                
                let _mc_run_outcome = elevator_control.run_machine(&ElevatorFSMInputs::Stop);
                
                info!("Event (Stop({})), was on floor: ({}), Transition(State: ({:?}), Outcome: ({:?}))", 
                        at_floor,
                        elevator_control.carriage_data.where_is(),
                        _mc_run_outcome.0, 
                        _mc_run_outcome.1
                    )
                    ;
                match _mc_run_outcome {
                    (ElevatorFSMStates::NonOperational, Some(ElevatorFSMOutputs::ExitPassengers)) => {
                        elevator_control.on_arrival();
                        elevator_control
                        .op_informant_channel
                        .as_ref()
                        .map(|channel| {
                            channel.send(ElevatorVocabulary::OpenTheDoor(at_floor)).unwrap();
                        });
                        elevator_control.pulley_actor.send_message(PulleyVocabulary::PowerOff).unwrap();

                    },
                    (_, Some(ElevatorFSMOutputs::Reached)) => {
                        elevator_control.on_arrival();
                        elevator_control
                        .op_informant_channel
                        .as_ref()
                        .map(|channel| {
                            channel.send(ElevatorVocabulary::OpenTheDoor(at_floor)).unwrap();
                        });
                    },
                    (_, None)   => { 
                        info!("State {:?}, Transition outcome {}", _mc_run_outcome.0, "is NOP")
                    },

                    (_, _) => {}
                }

            },

            ElevatorVocabulary::DoorClosed(at_floor) => {
                let _mc_run_outcome = elevator_control.run_machine(&ElevatorFSMInputs::DoorClosed);
                info!("Event (DoorClosed({})), was on floor: ({}), Transition(State: ({:?}), Outcome: ({:?}))", 
                    at_floor,
                    elevator_control.carriage_data.where_is(),
                    _mc_run_outcome.0, 
                    _mc_run_outcome.1
                )
                ;
                match _mc_run_outcome {
                    (ElevatorFSMStates::PoweredOff,_) => {
                        info!("Elevator is being powered off. Will be out of service!");
                        elevator_control
                        .op_informant_channel
                        .as_ref()
                        .map(|channel| {
                            channel.send(ElevatorVocabulary::ElevatorOutOfService).unwrap();
                        });
                        myself.stop(Some("Power off".to_owned()));
                    }
                    (_, Some(ElevatorFSMOutputs::CheckNextDest)) => {
                        match elevator_control.on_checking_next_dest() {
                            Some(NextDestTodo::EmergencyAtGroundFloorAlready) => {
                                // The operator has instructed for an emergency shutdown. We have to 
                                // prepare for the shutdown. 
                                // If it is already at ground floor, 
                                //     then we have to stop the operation.
                                info!("Emergency stop request, detected! Stoppin at ground floor.");
                                let _ = elevator_control.run_machine(&ElevatorFSMInputs::SwitchOff);
                                elevator_control
                                .op_informant_channel
                                .as_ref()
                                .map(|channel| {
                                    channel.send(ElevatorVocabulary::Stop(0)).unwrap();
                                });

                                elevator_control.set_next_destination(0);

                                myself.send_message(Stop(0)).unwrap();
                                
                            },
                            Some(NextDestTodo::EmergencyGotoGroundFloorNow) => {
                                // If it is not at the ground floor already, 
                                //     then we have to send the carriage to the gruond floor (floor = 0)
                                info!("Emergency stop request, detected! Moving to ground floor.");
                                let _ = elevator_control.run_machine(&ElevatorFSMInputs::SwitchOff);
                                elevator_control.set_next_destination(0);
                                elevator_control
                                .op_informant_channel
                                .as_ref()
                                .map(|channel| {
                                    channel.send(ElevatorVocabulary::MoveToGroundFloor).unwrap();
                                });
                                myself.send_message(MoveToFloor(0)).unwrap();
                                
                            },
                            Some(NextDestTodo::AllFineGotoNextPassenger(dest_floor)) => {
                                // While door is being closed, requests from passengers in other floors may 
                                // arrive (and be enqueued). Moreover, zero or more such requests may already
                                // in the queue. The earliest such request, if exists, must be handled. 
                                // So, the FSM is engaged and the state is changed. In addition to this, 
                                // the pulley is instructed to begin operation (up or down).
                                
                                info!("Next passenger request, at floor {}", dest_floor);
                                let _ = elevator_control
                                        .run_machine(&ElevatorFSMInputs::MoveTo(dest_floor));
                                elevator_control.set_next_destination(dest_floor);

                                elevator_control
                                .pulley_actor
                                .send_message(PulleyVocabulary::MoveToFloor(dest_floor))
                                .unwrap();
                                
                                elevator_control
                                .op_informant_channel
                                .as_ref()
                                .map(|channel| {
                                    channel.send(ElevatorVocabulary::MovingTo(dest_floor)).unwrap();
                                });
                                
                            },
                            None => {
                                info!("Staying at current floor {}", 
                                            elevator_control.carriage_data.where_is());
                                elevator_control
                                .op_informant_channel
                                .as_ref()
                                .map(|channel| {
                                    channel.send(ElevatorVocabulary::Stay(at_floor)).unwrap();
                                });
                            }
                        }
                    },
                    (_, _) => {},   
                }

            },

            _ => info!("Unknown message received by Elevator_Service"),
        };
       
        Ok(())
    }
}

mod test {

    use ractor::{Actor, ActorStatus};
    use tokio::{sync::mpsc, time::timeout};
    use tracing::info;

    use crate::{
        conversation::vocabulary::ElevatorVocabulary, 
        elevator_installation::pulley_machinery::PulleyActor, elevator_installation::elevator_service::PassengerLiftActor
    };
    use tokio::time::Duration;
    use assertx::assert_contains_exactly;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn when_elevator_is_powered_on_then_the_carriage_must_move_to_floor_zero() -> () {

        let (tx, mut rx) = mpsc::unbounded_channel();
        let floor_setting = vec! [
            (0.0,5.0), (0.0,10.0), (0.0,15.0), (0.0,20.0) // x never changes between floors
        ];
    
        let (pulley_ref, pulley_handle) = Actor::spawn(
                                Some(String::from("Test_pulley_actor-10")), 
                                PulleyActor, 
                                floor_setting
                            )
                            .await
                            .expect("Failed to create Pulley actor")
                            ;

        let (elev_ref, elev_handle) = 
            Actor::spawn(
                Some(String::from("Elevator-Actor-10")),
                PassengerLiftActor,
                (8,Some(tx.clone()),pulley_ref.clone())
            ).await
            .expect("Failed to start actor"); 

        // TODO: May be, it makes sense to use 'call_t!' macro to validate the actor's 
        // responses, in a synchronous fashion.    
        // let resp1 = call_t!(actor_ref,ElevatorVocabulary::PowerOn,1).unwrap();

        elev_ref.send_message(ElevatorVocabulary::PowerOn).unwrap();

        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::MoveToGroundFloor);
        }
        elev_ref.send_message(ElevatorVocabulary::PowerOff).unwrap();

        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::MovingTo(0));
        }

        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::Stop(0));
        }

        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::OpenTheDoor(0));
        }

        elev_ref.send_message(ElevatorVocabulary::DoorClosed(0)).unwrap();

        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::ElevatorOutOfService);
        }

        pulley_handle.await.unwrap();
        elev_handle.await.unwrap();

        loop {
            let st = pulley_ref.get_status();
            info!("Pulley status {:?}", st);
            if st == ActorStatus::Stopped { break; }
        }

        drop(tx);

    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn when_passenger_calls_then_the_pulley_must_displace_the_carriage_as_needed() {
        let (tx,mut rx) = mpsc::unbounded_channel();
        let floor_setting = vec! [
            (0.0,0.0), (0.0,5.0), (0.0,10.0), (0.0,15.0) // x never changes between floors
        ];
    

        let (pulley_ref, pulley_handle) = Actor::spawn(
                                Some(String::from("Test_pulley_actor-20")), 
                                PulleyActor, 
                                floor_setting.clone()
                            )
                            .await
                            .expect("Failed to create Pulley actor")
                            ;

        let (elev_ref, elev_handle) = 
            Actor::spawn(
                Some(String::from("Elevator-Actor-20")),
                PassengerLiftActor,
                (8,Some(tx.clone()),pulley_ref.clone())
            ).await
            .expect("Failed to start actor");

        elev_ref.send_message(ElevatorVocabulary::PowerOn).unwrap();
        // Let the main thread wait long enough for elevator to respond
        if let Some(msg_received) = 
                tokio::time::timeout(Duration::from_secs(1), rx.recv())
                .await
                .expect("timeout before messages are received from elevator") {
            assert_eq!(msg_received,ElevatorVocabulary::MoveToGroundFloor);
        }

        elev_ref.send_message(ElevatorVocabulary::MoveToFloor(3)).unwrap();
        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::MovingTo(3));
        }

        // Now, expect a series of messages indicating the current position, originally from the PulleyActor
        // Accumulate all of them.
        let mut actual_messages: Vec<ElevatorVocabulary> = Vec::new();
        while let Ok(msg_received) = 
            tokio::time::timeout(Duration::from_secs(2), rx.recv()).await{                         
                    actual_messages.push(msg_received.unwrap());        
        }

        let mut expected_messages: Vec<ElevatorVocabulary> =  
                    (1..=15)   // Hard-coded, because the floor_setting are pre-determinted.
                    .into_iter()
                    .map(|next_y| ElevatorVocabulary::CurrentCarriagePosn((0.0,next_y as f64)))
                    .collect()
                    ;
        expected_messages.push(ElevatorVocabulary::OpenTheDoor(3));        
        
        assert_contains_exactly!(actual_messages, expected_messages);

        elev_ref.send_message(ElevatorVocabulary::DoorClosed(3)).unwrap(); 
        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::Stay(3));
        }

        elev_ref.send_message(ElevatorVocabulary::PowerOff).unwrap();
        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::MovingTo(0));
        }

        expected_messages.clear();
        actual_messages.clear();

        expected_messages.push(ElevatorVocabulary::MovingTo(0));
    
        expected_messages.extend(
                (0..15).rev()   // Hard-coded, because the floor_setting are pre-determinted.
                .into_iter()
                .map(|next_y| ElevatorVocabulary::CurrentCarriagePosn((0.0,next_y as f64)))
                .collect::<Vec<ElevatorVocabulary>>()
        );
        expected_messages.push(ElevatorVocabulary::OpenTheDoor(0));

        while let Ok(msg_received) = 
            tokio::time::timeout(Duration::from_secs(3), rx.recv()).await{            
                    actual_messages.push(msg_received.unwrap());
        }

        assert_contains_exactly!(actual_messages, expected_messages);

        elev_ref.send_message(ElevatorVocabulary::DoorClosed(0)).unwrap();

        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::ElevatorOutOfService);
        }

        rx.close(); 

        pulley_handle.await.unwrap();

        loop {
            let st = pulley_ref.clone().get_status();
           info!("Pulley status {:?}", st);
           if st == ActorStatus::Stopped { break; }
        }

        elev_handle.await.unwrap();

        drop(tx);

    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn when_elev_is_moving_then_an_emergency_op_must_be_handled_after_reaching_dest() {
        let (tx,mut rx) = mpsc::unbounded_channel();
        let floor_setting = vec! [
            (0.0,0.0), (0.0,5.0), (0.0,10.0), (0.0,15.0) // x never changes between floors
        ];
    

        let (pulley_ref, pulley_handle) = Actor::spawn(
                                Some(String::from("Test_pulley_actor-21")), 
                                PulleyActor, 
                                floor_setting.clone()
                            )
                            .await
                            .expect("Failed to create Pulley actor")
                            ;

        let (elev_ref, elev_handle) = 
            Actor::spawn(
                Some(String::from("Elevator-Actor-21")),
                PassengerLiftActor,
                (8,Some(tx.clone()),pulley_ref.clone())
            ).await
            .expect("Failed to start actor");

        elev_ref.send_message(ElevatorVocabulary::PowerOn).unwrap();
        // Let the main thread wait long enough for elevator to respond
        if let Some(msg_received) = 
                tokio::time::timeout(Duration::from_secs(1), rx.recv())
                .await
                .expect("timeout before messages are received from elevator") {
            assert_eq!(msg_received,ElevatorVocabulary::MoveToGroundFloor);
        }

        elev_ref.send_message(ElevatorVocabulary::MoveToFloor(3)).unwrap();
        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::MovingTo(3));
        }

        let elev_ref_clone = elev_ref.clone();

        tokio::spawn( async move {
            // Give some time to pulley, so that it is moving, nice and steady!
            tokio::time::sleep(Duration::from_secs(2)).await;
            // Then, send PowerOff to the elevator, because of some emergency
            elev_ref_clone.send_message(ElevatorVocabulary::PowerOff).unwrap();

        })
        .await
        .unwrap();

        // Now, expect a series of messages indicating the current position, originally from the PulleyActor
        // Accumulate all of them.
        let mut actual_messages: Vec<ElevatorVocabulary> = Vec::new();
        while let Ok(msg_received) = 
            tokio::time::timeout(Duration::from_secs(2), rx.recv()).await{             
                    actual_messages.push(msg_received.unwrap());        
        }

        let mut expected_messages: Vec<ElevatorVocabulary> =  
                    (1..=15)   // Hard-coded, because the floor_setting are pre-determinted.
                    .into_iter()
                    .map(|next_y| ElevatorVocabulary::CurrentCarriagePosn((0.0,next_y as f64)))
                    .collect()
                    ;
        expected_messages.push(ElevatorVocabulary::OpenTheDoor(3));        
        
        assert_contains_exactly!(expected_messages, actual_messages);

        actual_messages.clear();      // Forget earlier messages;
        expected_messages.clear();    // Forget earlier messages;

        // Because an emergency PowerOff had been sent already, the elevator must automatically move to
        // ground floor (floor = 0), after the door is closed at the floor just reached (the door was 
        // opened). Therefore, messages from the elevator should be:
        // 1. MoveToFloor(0)
        // 2. Several CurrentCarriagePosn(x,y)
        // 3. OpenTheDoor(0)

        expected_messages.push(ElevatorVocabulary::MoveToGroundFloor);
        expected_messages.push(ElevatorVocabulary::MovingTo(0));
        let mut expected_position_messages: Vec<ElevatorVocabulary> = 
            (0..15).rev() // Hard-coded, because the floor_setting are pre-determinted, and decreasing
            .into_iter()
            .map(|next_y| ElevatorVocabulary::CurrentCarriagePosn((0.0,next_y as f64)))
            .collect()
            ;
        expected_messages.append(&mut expected_position_messages);
        expected_messages.push(ElevatorVocabulary::OpenTheDoor(0));  

        elev_ref.send_message(ElevatorVocabulary::DoorClosed(3)).unwrap(); 
        while let Ok(msg_received) = 
            tokio::time::timeout(Duration::from_secs(20), rx.recv()).await{                       
                    actual_messages.push(msg_received.unwrap());        
        }
    
        assert_contains_exactly!(actual_messages,expected_messages);

        info!("asserted here");

        elev_ref.send_message(ElevatorVocabulary::DoorClosed(0)).unwrap();
        if let Some(msg_received) = 
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout before messages are received from elevator") {
                assert_eq!(msg_received,ElevatorVocabulary::ElevatorOutOfService);
        }

        rx.close();
        loop {
            let st = pulley_ref.clone().get_status();
           info!("Pulley status {:?}", st);
           if st == ActorStatus::Stopped { break; }
        }

        pulley_handle.await.unwrap();

        

        elev_handle.await.unwrap();

        drop(tx);

    }
}

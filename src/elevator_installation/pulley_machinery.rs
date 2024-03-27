use async_trait::async_trait;
use ractor::time::send_after;
use ractor::concurrency::Duration;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use tracing::info;

use crate::conversation::vocabulary::{ElevatorVocabulary, PulleyVocabulary};

// use crate::elevator_service::ElevatorVocabulary::{self, MoveToFloor, Stop};

#[derive(Debug)]
struct FloorData {
    current: u8,
    destination: u8
}

impl FloorData {
    pub fn going_up(&self) -> bool {
        self.current < self.destination
    }

    pub fn going_down(&self) -> bool {
        self.current > self.destination
    }
}


#[derive(Debug)]
pub struct PulleyData {
    dest_posn: (f64,f64),
    current_posn: (f64,f64),
    displacement: f64,
    floor_data: FloorData,
    floors_to_position_map: Vec<(f64 /* row as x-coord */, f64 /* col as y-coord */)>,
    elevator_controller_actor: Option<ActorRef<ElevatorVocabulary>>

}

impl PulleyData {
    pub fn new(floors_to_position_map: Vec<(f64, f64)>) -> Self {

        let currently_at = 0;
        let going_to     = 0; // At the start, pulley doesn't need to know
        let start_posn = floors_to_position_map[0];
        PulleyData { 
            dest_posn: start_posn, 
            current_posn: start_posn, 
            displacement: 0.0,
            floor_data: FloorData { current: currently_at, destination: going_to },
            floors_to_position_map,
            elevator_controller_actor: None
        }
    }

    pub fn hook(&mut self, controller: ActorRef<ElevatorVocabulary>) -> &mut Self {
        self.elevator_controller_actor = Some(controller);
        self
    }

    pub fn prepare_for_moving(&mut self, dest_f: u8) -> &mut Self {

        if self.floor_data.current == dest_f { self }
        else {
            self.floor_data.destination = dest_f; 
            self.current_posn = self.floors_to_position_map[self.floor_data.current as usize];
            self.dest_posn    = self.floors_to_position_map[self.floor_data.destination as usize];
            if self.floor_data.going_up() {
                self.displacement = 1.0; // Because the pulley pulls the carriage upwards
            }
            else {
                self.displacement = -1.0; // Because the pulley releases the carriage downwards.
            };

            self
        }
    }

    pub fn on_pulley_moving(&mut self) -> &mut Self {
        self.current_posn.1 += self.displacement;
        self
    }

    pub fn has_reached_dest(&self) -> bool {
        self.current_posn.1 == self.dest_posn.1
    }

    pub fn adjust_floor_data(&mut self) -> &mut Self {
        let currently_at = self.floor_data.destination;
        let adjusted_floor_data = FloorData { current: currently_at, destination: self.floor_data.destination };
        self.floor_data = adjusted_floor_data;
        self
    }

    pub fn currently_at(&self) -> (u8,(f64,f64)) {
        let current_floor = self.floor_data.current;
        (current_floor,(self.floors_to_position_map[current_floor as usize]))
    }
}

pub struct PulleyActor;

#[async_trait]
impl Actor for PulleyActor {
    type Msg       = PulleyVocabulary;
    type State     = PulleyData;
    type Arguments = Vec<(f64 /* x at start */, f64 /* y at start */)>;

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, args: Self::Arguments) -> 
        Result<Self::State, ActorProcessingErr> {
            Ok(PulleyData::new(args))
    }

    async fn post_start( &self,_myself: ActorRef<Self::Msg>, data: &mut Self::State) ->
        Result<(), ActorProcessingErr> {
            info!("floors as seen by pulley {:?}", data.floors_to_position_map);
            Ok(())
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        carriage: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {

        match message {
           PulleyVocabulary::PowerOn(controller) => {
                carriage.hook(controller);
                info!("Pulley: is being powered on!");
           },
           PulleyVocabulary::PowerOff => {
            // TODO: Abort timer, if it is still working
            info!("Pulley: is being powered off");
            myself.stop(Some(String::from("powered down")));
           },
           PulleyVocabulary::MoveToFloor(f) => {
            info!("Pulley: needs to move to floor({})", f);
             carriage.prepare_for_moving(f);
             send_after(
                Duration::from_millis(100),
                myself.get_cell(),
                || { PulleyVocabulary::PulleyHasMoved }
            );
           },
           PulleyVocabulary::PulleyHasMoved => {
                carriage.on_pulley_moving();
                /* info!("Pulley: is passing by {},{}, dest_y {}",
                                carriage.current_posn.0,
                                carriage.current_posn.1,
                                carriage.dest_posn.1
                            ); */
                carriage
                .elevator_controller_actor
                .as_ref()
                .map(|controller| {
                    controller.send_message(
                        ElevatorVocabulary::CurrentCarriagePosn(
                            (carriage.current_posn.0,carriage.current_posn.1))
                        ).unwrap();
                });

                if !carriage.has_reached_dest() {
                    send_after(
                        Duration::from_millis(100),
                        myself.get_cell(),
                        || { PulleyVocabulary::PulleyHasMoved }
                    ); 
                }
                else {
                    
                    carriage.adjust_floor_data();

                    info!("Pulley: stops at destination, dest_y {}, currently at floor {}, floor_y {}",
                        carriage.dest_posn.1,
                        carriage.currently_at().0,
                        carriage.currently_at().1.1
                    );
                    
                    carriage.
                    elevator_controller_actor
                    .as_ref()
                    .map(|controller| {
                        controller.send_message(ElevatorVocabulary::Stop(carriage.floor_data.current))
                        .unwrap()
                    }).unwrap();
                }   

            }
        };
       
        Ok(())
    }
}

mod test {

    use ractor::Actor;
    use tokio::sync::mpsc;
    use super::{PulleyActor, PulleyData};
     
    #[tokio::test]
    async fn when_powerd_on_then_should_be_on_0th_floor() -> () {
        let floor_setting = vec! [
            (0.0,0.0), (0.0,5.0), (0.0,10.0), (0.0,15.0) // x never changes between floors
        ];
    

        let pulley_data = PulleyData::new(floor_setting);

        let current_status = pulley_data.currently_at();

        assert_eq!(current_status.0, 0);
        assert_eq!(current_status.1.0, 0.0);
        assert_eq!(current_status.1.1, 0.0);


    }

    #[tokio::test]
    async fn when_moving_to_a_floor_then_pulley_must_know_it_has_reached() -> () {
        let floor_setting = vec! [
            (0.0,0.0), (0.0,5.0), (0.0,10.0), (0.0,15.0) // x never changes between floors
        ];
    

        let pulley_data = &mut PulleyData::new(floor_setting.clone());

        let current_status = pulley_data.currently_at();

        pulley_data.prepare_for_moving(2);

        let pulley_needs_to_move_by = floor_setting[2].1 - floor_setting[0].1;

        for _next in 0 .. pulley_needs_to_move_by as u16 {
            pulley_data.on_pulley_moving();
        }

        assert_eq!(pulley_data.has_reached_dest(),true);

    }

    #[tokio::test]
    async fn when_moving_to_a_floor_then_pulley_must_emit_correct_y_displacement() -> () {
        let floor_setting = vec! [
            (0.0,0.0), (0.0,5.0), (0.0,10.0), (0.0,15.0) // x never changes between floors
        ];
    
        let pulley_data = &mut PulleyData::new(floor_setting.clone());

        // when going up

        pulley_data.prepare_for_moving(2);

        let expcted_carriage_posn_1_up: [f64; 10] = [1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0];

        let pulley_needs_to_move_by = floor_setting[2].1 - floor_setting[0].1;
        let mut actuals_carriage_posn_y: Vec<f64> = Vec::new();

        for _next in 0 .. pulley_needs_to_move_by as u16 {
            pulley_data.on_pulley_moving();
            actuals_carriage_posn_y.push(pulley_data.current_posn.1);
            
        }

        for next in expcted_carriage_posn_1_up.to_vec().into_iter().zip(actuals_carriage_posn_y.clone()).into_iter() {
            assert_eq!(next.0,next.1);
        }

        assert_eq!(pulley_data.has_reached_dest(),true);

        pulley_data.adjust_floor_data();


        // when going down
        pulley_data.prepare_for_moving(0); 

        let expcted_carriage_posn_1_down: [f64; 10] = [9.0,8.0,7.0,6.0,5.0,4.0,3.0,2.0,1.0,0.0];
        actuals_carriage_posn_y.clear();

        for _next in 0 .. pulley_needs_to_move_by as u16 {
            pulley_data.on_pulley_moving();
            actuals_carriage_posn_y.push(pulley_data.current_posn.1);
            
        }

        for next in expcted_carriage_posn_1_down.to_vec().into_iter().zip(actuals_carriage_posn_y.clone()).into_iter() {
            assert_eq!(next.0,next.1);
        }

    }
}
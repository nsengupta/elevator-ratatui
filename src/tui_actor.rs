use ratatui::prelude::Backend;


pub struct TuiMachinery <B: Backend> {
    pub tui: Tui<B>,
    pub pulley_actor: ActorRef<PulleyVocabulary>,
    pub elevator_actor: ActorRef<ElevatorVocabulary>
}


#[async_trait]
impl<B: Backend> Actor for Tui<B> {
    type Msg = ElevatorVocabulary;
    type State = (Tui<B>,ActorRef<PulleyVocabulary>,ActorRef<ElevatorVocabulary>); // Modify
    type Arguments = (Tui<B>,ActorRef<PulleyVocabulary>,ActorRef<ElevatorVocabulary>); // Modify

    async fn pre_start(&self, myself: ActorRef<Self::Msg>, args: Self::Arguments) ->
            Result<Self::State, ActorProcessingErr>   {
        Ok(args)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        tui_component: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        
        match message {
            TuiVocabulary::StartService => {
                cast! (tui_component.1, tui_component.0.get_carriage_displacement_map_per_floor((0.0,0.0)));
                send_after(
                    Duration::from_millis(10),
                    myself.get_cell(),
                    || { PingPongMessage::CheckAreTargetsAlive }
                );
            },

            TuiVocabulary::StopService  => todo!(),
            TuiVocabulary::CarriageToGroundFloor => {
                tui_component.0.ui.carriage.move_to_ground();
            },
            TuiVocabulary::CarriageMovesUp (displacement_x,displacement_y) => {
                tui_component.0.ui.carriage.move_carriage_up(displacement_x,displacement_y);
            },
            TuiVocabulary::CarriageMovesDown (displacement_x,displacement_y)=> {
                tui_component.0.ui.carriage.move_carriage_down(displacement_x,displacement_y);
            },
        }
    }
}

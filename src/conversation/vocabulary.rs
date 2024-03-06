
use ractor::ActorRef;

use ractor::RpcReplyPort;
use ractor_cluster::RactorMessage;


#[derive(RactorMessage)]
pub enum ElevatorVocabulary {
    DoorClosed(u8, ActorRef<ElevatorVocabulary>),
    OpenTheDoor(u8,ActorRef<ElevatorVocabulary>),
    MoveToFloor(u8, ActorRef<ElevatorVocabulary>),
    Stop(u8,ActorRef<ElevatorVocabulary>),
    Stay(u8,ActorRef<ElevatorVocabulary>),
    CurrentCarriagePosn((f64 /* x */, f64 /* y  */)),
    PowerOn(RpcReplyPort<ActorRef<ElevatorVocabulary>>),
    PowerOff(Option<ActorRef<ElevatorVocabulary>>),
    TempPulleyVocabularyGoto(u8,ActorRef<ElevatorVocabulary>)
}

#[derive(RactorMessage)]
pub enum TuiVocabulary {
    CarriageMovesUp   ((f64 /* x, remains unchanged */, f64 /* displacement */)),
    CarriageMovesDown ((f64 /* x, remains unchanged */, f64 /* displacement */)),
    CarriageToGroundFloor,
    StartService,
    StopService
}

#[derive(RactorMessage)]
pub enum PulleyVocabulary {
    SetupDisplacementMap(Vec<(f64,f64)>),
    OperatePulley(u16),
}

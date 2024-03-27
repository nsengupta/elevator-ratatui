use ractor::ActorRef;
use ractor_cluster::RactorMessage;

#[derive(RactorMessage)]
pub enum PulleyVocabulary {
    MoveToFloor(u8),
    PulleyHasMoved,
    PowerOn(ActorRef<ElevatorVocabulary>),
    PowerOff
}

#[derive(RactorMessage, Clone, Debug, PartialEq)]
pub enum ElevatorVocabulary {
    DoorClosed(u8),
    OpenTheDoor(u8),
    MoveToFloor(u8),
    Stop(u8),
    Stay(u8),
    CurrentCarriagePosn((f64 /* x */, f64 /* y  */)),
    PowerOn,
    PowerOff,
    MoveToGroundFloor, // emergency or regular shutdown
    MovingTo(u8), // for information, from elevator to operator
    ElevatorOutOfService
}



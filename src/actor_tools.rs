#[derive(Debug)]
pub enum MsgActor {
	Start,
	Stop,
	Pause,
	Ping,
	Pong,
	LogPrint(String),

}

#[derive(Debug, Copy, Clone)]
pub enum State {
	Started, Stopped
}

pub struct ActorState {
	pub a_state:crossbeam_utils::atomic::AtomicCell<State>
}


use crossbeam_utils::atomic::AtomicCell;
use std::time::{Duration, Instant};
use crossbeam_channel::{after, tick, Sender, Receiver};
use std::thread::{spawn, sleep};


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

pub struct Actor {

	// owned by main thread but cloneable via crossbeam_channel
	inbound_multi_producer:crossbeam_channel::Sender<MsgActor>,
	inbound_single_consumer:crossbeam_channel::Receiver<MsgActor>,
	outbound_multi_producer:crossbeam_channel::Sender<MsgActor>,
	outbound_single_consumer:crossbeam_channel::Receiver<MsgActor>,

}

impl Actor {
	pub fn new()-> Actor {
		let mut inbound_channel = crossbeam_channel::unbounded();
		let mut outbound_channel = crossbeam_channel::unbounded();
		let new_actor = Actor {
			inbound_multi_producer: inbound_channel.0,
			inbound_single_consumer: inbound_channel.1,
			outbound_multi_producer: outbound_channel.0,
			outbound_single_consumer: outbound_channel.1,
		};
		new_actor
	}

	// Give the main thread a way to send messages TO me
	pub fn get_sender(&self) -> Sender<MsgActor>{
		self.inbound_multi_producer.clone()
	}

	// Give me a way to send messages TO the main thread
	pub fn get_receiver(&self) -> Receiver<MsgActor>{
		self.outbound_single_consumer.clone()
	}

	pub fn run(&self){
		// this uses this to listen
		let single_consumer_clone = self.inbound_single_consumer.clone();

		let outbound_multiproducer = self.outbound_multi_producer.clone();


		/*

			Instead of retaining state in the primary object, create it here, in the "temporal"
			closure of the listen function then give it to the infinite comms loop. The main thread
			doesn't need access to it. Main thread can send a message here if it wants to know what
			it is.

			Do other functions need access to the state? They won't have it unless it's passed via
			message to/fro.

		 */
		let c_state = ActorState {
			a_state:AtomicCell::new(State::Stopped),
		};

		spawn(move ||{
			loop {
				match single_consumer_clone.recv() {
					Ok(m) => {
						//println!("[listen] receive ok: {:?}", m);
						match m {
							MsgActor::Start => {
								println!("[listen] received Message::Start");
								c_state.a_state.store(State::Started);
							},
							MsgActor::Stop => {
								println!("[listen] received Message::Stop");
								c_state.a_state.store(State::Stopped);
							},
							MsgActor::Ping => {

								use core::borrow::Borrow;
								println!("[listen] received Message::Ping; status {:?}",c_state.a_state.borrow().load());

								// TODO: send a message back
								outbound_multiproducer.send(MsgActor::Pong);

							}
							_ => {
								// ignore other messages
							}
							// exhaustive of enum options

						}
					},
					Err(_) => {
						println!("[listen] receive error")
					}
				}
			}
		});
		println!("[listen] listening")
	}
}



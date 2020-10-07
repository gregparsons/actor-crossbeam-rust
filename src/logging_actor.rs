

use crossbeam_utils::atomic::AtomicCell;
use std::time::{Duration, Instant};
use crossbeam_channel::{after, tick, Sender, Receiver};
use std::thread::{spawn, sleep};
use crate::actor::{MsgActor, ActorState, State};

pub struct Actor {

	name:String,
	inbound_multi_producer:crossbeam_channel::Sender<MsgActor>,
	inbound_single_consumer:crossbeam_channel::Receiver<MsgActor>,
	pub parent_tx:Sender<MsgActor>,
	//pub logger_tx:Sender<MsgActor>,

}

impl Actor {
	pub fn new(actor_name:String, parent_tx_new: Sender<MsgActor>) -> Actor {
		let mut inbound_channel = crossbeam_channel::unbounded();
		// let mut outbound_channel = crossbeam_channel::unbounded();
		let new_actor = Actor {
			name : actor_name,
			// listen on this:
			inbound_single_consumer: inbound_channel.1,
			// give out clones of this to anyone who wants to talk to us:
			inbound_multi_producer: inbound_channel.0,
			// talk to the operator on this:
			parent_tx : parent_tx_new,
			// logger_tx : logging_tx,
		};
		new_actor
	}

	// Give the main thread a way to send messages TO me
	pub fn get_sender(&self) -> Sender<MsgActor>{
		self.inbound_multi_producer.clone()
	}

	// // Give me a way to send messages TO the main thread
	// pub fn get_receiver(&self) -> Receiver<MessageRequest>{
	// 	self.outbound_single_consumer.clone()
	// }

	pub fn run(&self){
		// this uses this to listen
		let single_consumer_clone = self.inbound_single_consumer.clone();

		// let outbound_multiproducer = self.outbound_multi_producer.clone();


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
							MsgActor::LogPrint(msg) => {
								println!("[logging_actor] LogPrint: {}", &msg);
							}
							_ => {
								println!("[pinger_actor] Message: Unknown" );
							}

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



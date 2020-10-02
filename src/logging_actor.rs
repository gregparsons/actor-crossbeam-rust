

use crossbeam_utils::atomic::AtomicCell;
use std::time::{Duration, Instant};
use crossbeam_channel::{after, tick, Sender, Receiver};
use std::thread::{spawn, sleep};
use crate::actor::{MessageRequest, ActorState, State};

pub struct Actor {

	// owned by main thread but cloneable vi/**/a crossbeam_channel
	inbound_multi_producer:crossbeam_channel::Sender<MessageRequest>,
	inbound_single_consumer:crossbeam_channel::Receiver<MessageRequest>,
	outbound_multi_producer:crossbeam_channel::Sender<MessageRequest>,
	outbound_single_consumer:crossbeam_channel::Receiver<MessageRequest>,

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
	pub fn get_sender(&self) -> Sender<MessageRequest>{
		self.inbound_multi_producer.clone()
	}

	// Give me a way to send messages TO the main thread
	pub fn get_receiver(&self) -> Receiver<MessageRequest>{
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
							MessageRequest::Start => {
								println!("[listen] received Message::Start");
								c_state.a_state.store(State::Started);
							},
							MessageRequest::Stop => {
								println!("[listen] received Message::Stop");
								c_state.a_state.store(State::Stopped);
							},
							MessageRequest::LogPrint(msg) => {
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



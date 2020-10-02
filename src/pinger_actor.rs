
use crossbeam_utils::atomic::AtomicCell;
use std::time::{Duration, Instant};
use crossbeam_channel::{after, tick, Sender, Receiver};
use std::thread::{spawn, sleep};
use crate::actor::{MessageRequest, ActorState, State};

pub struct Actor {

	// owned by main thread but cloneable vi/**/a crossbeam_channel
	inbound_multi_producer:crossbeam_channel::Sender<MessageRequest>,
	inbound_single_consumer:crossbeam_channel::Receiver<MessageRequest>,

	// probably don't need this, just need parent's
	outbound_multi_producer:crossbeam_channel::Sender<MessageRequest>,
	outbound_single_consumer:crossbeam_channel::Receiver<MessageRequest>,

	parent_tx:crossbeam_channel::Sender<MessageRequest>,

}

impl Actor {
	pub fn new(parent_tx_new: crossbeam_channel::Sender<MessageRequest>)-> Actor {
		let mut inbound_channel = crossbeam_channel::unbounded();
		let mut outbound_channel = crossbeam_channel::unbounded();
		let new_actor = Actor {
			inbound_multi_producer: inbound_channel.0,
			inbound_single_consumer: inbound_channel.1,
			outbound_multi_producer: outbound_channel.0,
			outbound_single_consumer: outbound_channel.1,
			parent_tx : parent_tx_new,
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

	pub fn run(&self, is_ping:bool){
		// this uses this to listen
		let single_consumer_clone = self.inbound_single_consumer.clone();
		let outbound_multiproducer = self.outbound_multi_producer.clone();

		// Do Work Here
		// Spawns a pinger infinite loop
		// TODO: spawn as thread, send message to stop?
		if(is_ping) {
			// Should we send pings? If false, then just listen and send pongs.
			let parent_tx_clone = self.parent_tx.clone();
			{
				// Send a periodic PING expecting a PONG back
				let ticker: crossbeam_channel::Receiver<Instant> = tick(Duration::from_millis(500));
				let tx = outbound_multiproducer.clone();
				let ping_thread = spawn(move || {
					loop {
						crossbeam_channel::select! {
						recv(ticker) -> _ => {
							parent_tx_clone.send(MessageRequest::Ping);
						}
					}
					}
				});
			}
		}


		let parent_tx_clone2 = self.parent_tx.clone();
		spawn(move ||{
			loop {
				match single_consumer_clone.recv() {
					Ok(m) => {
						//println!("[listen] receive ok: {:?}", m);
						match m {
							MessageRequest::Start => {
								println!("[listen] received Message::Start");
								// c_state.a_state.store(State::Started);
							},
							MessageRequest::Stop => {
								println!("[listen] received Message::Stop");
								// c_state.a_state.store(State::Stopped);
							},
							MessageRequest::Ping => {
								// If you got a ping, log it, and reply with a pong.
								// println!("[listen] received ping, sending pong {:?}", MessageRequest::Ping);
								parent_tx_clone2.send(MessageRequest::LogPrint("[pinger] ping".to_string()));
								parent_tx_clone2.send(MessageRequest::Pong);

							},
							MessageRequest::Pong => {
								// If you got a pong, stop the madness and log it.
								// println!("[listen] received pong, sending log message");
								parent_tx_clone2.send(MessageRequest::LogPrint("[ponger] pong".to_string()));

							},

							_ => {
								println!("[pinger_actor] Message: Unknown" );
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



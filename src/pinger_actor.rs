
use crossbeam_utils::atomic::AtomicCell;
use std::time::{Duration, Instant};
use crossbeam_channel::{after, tick, Sender, Receiver};
use std::thread::{spawn, sleep};
use crate::actor::{MessageRequest, ActorState, State};

pub struct Actor {
	inbound_multi_producer:crossbeam_channel::Sender<MessageRequest>,
	inbound_single_consumer:crossbeam_channel::Receiver<MessageRequest>,
	pub parent_tx:crossbeam_channel::Sender<MessageRequest>,
}

impl Actor {
	pub fn new(parent_tx_new: crossbeam_channel::Sender<MessageRequest>)-> Actor {

		let mut inbound_channel = crossbeam_channel::unbounded();

		let new_actor = Actor {
			// listen on this:
			inbound_single_consumer: inbound_channel.1,

			// give out clones of this to anyone who wants to talk to us:
			inbound_multi_producer: inbound_channel.0,

			// talk to the operator on this:
			parent_tx : parent_tx_new,
		};
		new_actor
	}

	// Give the main thread a way to send messages TO me
	pub fn get_sender(&self) -> Sender<MessageRequest>{
		self.inbound_multi_producer.clone()
	}

	pub fn run(&self, is_ping:bool){
		self.do_one_time_work(is_ping);
		self.listen();
	}
	
	fn do_one_time_work(& self, is_ping:bool){
		// Do Work Here
		// Spawns a pinger infinite loop
		if(is_ping) {
			{
				// Send a periodic PING expecting a PONG back
				let ticker: crossbeam_channel::Receiver<Instant> = tick(Duration::from_millis(500));
				let parent_tx_clone = self.parent_tx.clone();
				let ping_thread = spawn(move || {
					loop {

						crossbeam_channel::select! {
							recv(ticker) -> _ => {
								//self::parent_tx.send(MessageRequest::Ping);
								parent_tx_clone.send(MessageRequest::Ping);

							}
						}
					}
				});
			}
		}
	}

	fn listen(&self){
		let rx = self.inbound_single_consumer.clone();
		let parent_tx = self.parent_tx.clone();
		spawn(move ||{
			loop {
				match rx.recv() {
					Ok(m) => {
						//println!("[listen] receive ok: {:?}", m);
						match m {
							MessageRequest::Start => {
								println!("[listen] received Message::Start");
								// c_state.a_state.store(State::Started);
							},
							MessageRequest::Stop => {
								println!("[listen] received Message::Stop");
								return;
							},
							MessageRequest::Ping => {
								// If you got a ping, log it, and reply with a pong.
								// println!("[listen] received ping, sending pong {:?}", MessageRequest::Ping);
								parent_tx.send(MessageRequest::LogPrint("[pinger] ping".to_string()));
								parent_tx.send(MessageRequest::Pong);
							},
							MessageRequest::Pong => {
								// If you got a pong, stop the madness and log it.
								// println!("[listen] received pong, sending log message");
								parent_tx.send(MessageRequest::LogPrint("[ponger] pong".to_string()));
							},
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




use std::time::{Duration, Instant};
use crossbeam_channel::{Sender, Receiver};
use std::thread::{spawn};
use crate::actor_tools::MsgActor;

pub struct Actor {

	name:String,
	inbound_multi_producer:crossbeam_channel::Sender<MsgActor>,
	inbound_single_consumer:crossbeam_channel::Receiver<MsgActor>,
	pub parent_tx:Sender<MsgActor>,
	pub logger_tx:Sender<MsgActor>,

}

impl Actor {

	pub fn new (actor_name: String, parent_tx_new: Sender<MsgActor>, logging_tx:Sender<MsgActor>) -> Actor {
		let inbound_channel = crossbeam_channel::unbounded();
		let new_actor = Actor {
			name : actor_name,
			// listen on this:
			inbound_single_consumer: inbound_channel.1,
			// give out clones of this to anyone who wants to talk to us:
			inbound_multi_producer: inbound_channel.0,
			// talk to the operator on this:
			parent_tx : parent_tx_new,
			logger_tx : logging_tx,
		};
		new_actor
	}

	pub fn run(&self, is_ping:bool){
		self.do_one_time_work(is_ping);
		self.listen();
	}

	// Give the main thread a way to send messages TO me
	pub fn get_sender(&self) -> Sender<MsgActor>{
		self.inbound_multi_producer.clone()
	}

	fn do_one_time_work(& self, is_ping:bool){
		if is_ping  {
			{
				// Send a periodic PING expecting a PONG back
				let ticker: Receiver<Instant> = crossbeam_channel::tick(Duration::from_millis(500));
				let parent_tx_clone = self.parent_tx.clone();
				spawn(move || {
					loop {
						crossbeam_channel::select! {
							recv(ticker) -> _ => {
								//self::parent_tx.send(MessageRequest::Ping);
								let _ = parent_tx_clone.send(MsgActor::Ping);

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
		let logger_tx = self.logger_tx.clone();
		spawn(move ||{
			loop {
				match rx.recv() {
					Ok(m) => {
						//println!("[listen] receive ok: {:?}", m);
						match m {
							MsgActor::Start => {
								println!("[listen] received Message::Start");
								// c_state.a_state.store(State::Started);
							},
							MsgActor::Stop => {
								println!("[listen] received Message::Stop");
								return;
							},
							MsgActor::Ping => {
								// If you got a ping, log it, and reply with a pong.
								// println!("[listen] received ping, sending pong {:?}", MessageRequest::Ping);


								let _ = logger_tx.send(MsgActor::LogPrint("[pinger-direct-to-logger] ping".to_string()));
								let _ = parent_tx.send(MsgActor::LogPrint("[pinger] ping".to_string()));
								let _ = parent_tx.send(MsgActor::Pong);
							},
							MsgActor::Pong => {
								// If you got a pong, stop the madness and log it.
								// println!("[listen] received pong, sending log message");
								let _ = parent_tx.send(MsgActor::LogPrint("[ponger] pong".to_string()));
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



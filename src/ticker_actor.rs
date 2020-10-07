

use crossbeam_utils::atomic::AtomicCell;
use std::time::{Duration, Instant};
use crossbeam_channel::{after, tick, Sender, Receiver};
use std::thread::{spawn, sleep};
use crate::actor::{MsgActor, ActorState, State};

pub struct Actor {

	name:String,
	inbound_multi_producer:crossbeam_channel::Sender<MsgActor>,
	inbound_single_consumer:crossbeam_channel::Receiver<MsgActor>,
	parent_tx:Sender<MsgActor>,
	logger_tx:Sender<MsgActor>,
}

impl Actor {
	pub fn new(actor_name:String, parent_tx_new: Sender<MsgActor>, logger:Sender<MsgActor>) -> Actor {
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
			logger_tx : logger,
		};
		new_actor
	}

	// Give the main thread a way to send messages TO me
	pub fn get_sender(&self) -> Sender<MsgActor>{
		self.inbound_multi_producer.clone()
	}

	pub fn run(&self){
		// this uses this to listen
		let single_consumer_clone = self.inbound_single_consumer.clone();

		let c_state = ActorState {
			a_state:AtomicCell::new(State::Stopped),
		};

		// ticker_tx needs to be available from outside the loop and outside the thread, rx gets
		// cloned and moved inside the spawned thread to receive thread control messages
		let (ticker_tx, ticker_rx) = crossbeam_channel::unbounded();
		let logger_tx = self.logger_tx.clone();

		let mut should_restart = true;

		spawn(move ||{

			// Restart HTTP Connection
			loop {

				// Work

				if should_restart {
					println!("[ticker_controller] spawning ticker_loop_thread");

					let url_sandbox = "wss://ws-feed-public.sandbox.pro.coinbase.com".to_string(); //std::env::var("COINBASE_URL").expect("COINBASE_URL must be set");
					let url_pro = "wss://ws-feed.pro.coinbase.com".to_string(); // std::env::var("COINBASE_URL").expect("COINBASE_URL must be set");
					let url = url_sandbox;

					// WebSocket Thread
					// Copy stuff for the move into the thread
					let url = url.clone();
					let logger_tx = logger_tx.clone();
					let ticker_rx = ticker_rx.clone();

					// Spawn
					std::thread::spawn(move ||
						start_ticker_websocket(
							url,
							ticker_rx,
							logger_tx
						)
					);

					println!("[ticker_controller] spawned ticker_loop_thread, waiting for message...");
				}


				// Comms

				match single_consumer_clone.recv() {
					Ok(m) => {
						match m {
							MsgActor::Start => {
								println!("[listen] received Message::Start");
								c_state.a_state.store(State::Started);
								should_restart = true;
							},
							MsgActor::Stop => {
								println!("[listen] received Message::Stop");
								c_state.a_state.store(State::Stopped);
								ticker_tx.send(MsgActor::Stop);
								should_restart = false;
								return;
							},
							MsgActor::Pause => {
								println!("[listen] received Message::Pause");
								ticker_tx.send(MsgActor::Stop);
								should_restart = false;
							}
							// MsgActor::LogPrint(msg) => {
							// 	println!("[logging_actor] LogPrint: {}", &msg);
							// }
							_ => {}
						}
					},
					_ => {}
				}
			}
		});
		println!("[listen] listening")
	}
}

fn start_ticker_websocket(url:String, rx: Receiver<MsgActor>, logger: Sender<MsgActor>)->() {

	fn generate_subscribe_message() -> json::JsonValue {

		let subscribe_message = json::object!{
			// quotes on keys are optional
			"type": "subscribe",
			"product_ids": ["BTC-USD"],
			"channels": ["heartbeat", "ticker"]
		};
		subscribe_message.to_owned()
	}

	// let url = std::env::var("COINBASE_URL").expect("COINBASE_URL must be set");

	match url::Url::parse(&url){
		Err(e) => println!("[ticker] parse(&url) failed: {}", &e),
		Ok(url) => {
			let mut do_restart = true;

			// Restart HTTP upgrade
			while do_restart {

				// Http Upgrade Request
				let websocket_result = tungstenite::connect(&url);

				match websocket_result {
					Err(e) => {
						// TODO: loop and keep on trying
						println!("[main] websocket connection failed, trying in 2 seconds: {}", &e);
						std::thread::sleep(std::time::Duration::from_millis(2000));
						do_restart = true;
					},
					Ok((mut socket, _)) => {
						println!("[main] websocket connected");

						// Subscribe to Coinbase WebSocket
						let _ = socket.write_message(tungstenite::Message::Text(generate_subscribe_message().dump()));

						// WebSocket
						loop {
							let ws_result = socket.read_message();
							match ws_result {
								Err(e) => {
									match e {
										tungstenite::error::Error::ConnectionClosed => {
											// https://docs.rs/tungstenite/0.11.1/tungstenite/error/enum.Error.html#variant.ConnectionClosed
											// TODO: stop the loop; attempt to reopen socket
											println!("[parse_incoming_socket_blocking] socket: Error::ConnectionClosed");

											// break from the websocket loop, fall back to the http loop
											break;
											do_restart = true;
										},
										_ => {
											println!("[parse_incoming_socket_blocking] socket read failed: {}", &e);

											// break from the websocket loop, fall back to the http loop
											break;
											do_restart = true;
										}
									}
								},
								Ok(ws_mesg) => {
									match ws_mesg {
										tungstenite::Message::Text(t) => {
											// println!("[ticker] {}", &t);
											// TODO: database here
											logger.send(MsgActor::LogPrint(format!("[ticker via logger] {}", &t)));

										},
										_ => println!("[main] unknown socket message or something not text"),
									}
								}
							}


							// Thread command and control

							match rx.try_recv() {
								Ok(MsgActor::Stop) => {
									println!("[ticker] message: stop");
									return;
								},
								_ => {},
							}
						}
					}
				}
			}
		}
	}
}


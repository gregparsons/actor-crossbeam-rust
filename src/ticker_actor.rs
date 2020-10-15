

use crossbeam_utils::atomic::AtomicCell;
use crossbeam_channel::{Sender, Receiver};
use crate::actor_tools::{MsgActor, Ticker2};
use crate::actor_tools;

pub struct Actor {

	name:String,
	inbound_multi_producer:crossbeam_channel::Sender<MsgActor>,
	inbound_single_consumer:crossbeam_channel::Receiver<MsgActor>,
	parent_tx:Sender<MsgActor>,
	logger_tx:Sender<MsgActor>,
	db_tx:Sender<MsgActor>,
	url_string:String,
	is_pro:bool,
}

impl Actor {
	pub fn clone(&self) -> Actor {
		let copy = Actor {
			name:self.name.clone(),
			inbound_multi_producer:self.inbound_multi_producer.clone(),
			inbound_single_consumer:self.inbound_single_consumer.clone(),
			parent_tx:self.parent_tx.clone(),
			logger_tx:self.logger_tx.clone(),
			db_tx:self.db_tx.clone(),
			url_string:self.url_string.clone(),
			is_pro:self.is_pro,
		};
		copy
	}

	pub fn new(actor_name:String, url_string:String, is_pro:bool, parent_tx_new: Sender<MsgActor>, logger:Sender<MsgActor>, db_tx:Sender<MsgActor>) -> Actor {
		let inbound_channel = crossbeam_channel::unbounded();
		let new_actor = Actor {
			name : actor_name,
			// listen on this:
			inbound_single_consumer: inbound_channel.1,
			// give out clones of this to anyone who wants to talk to us:
			inbound_multi_producer: inbound_channel.0,
			// talk to the operator on this:
			parent_tx : parent_tx_new,
			logger_tx : logger,
			db_tx : db_tx,
			url_string : url_string,
			is_pro:is_pro,
		};
		new_actor
	}

	// Give the main thread a way to send messages TO me
	pub fn get_sender(&self) -> Sender<MsgActor>{
		self.inbound_multi_producer.clone()
	}

	pub fn run(&self){
		// this uses this to listen
		let name = self.name.clone();
		let single_consumer_clone = self.inbound_single_consumer.clone();

		// let c_state = ActorState {
		// 	a_state:AtomicCell::new(State::Stopped),
		// };

		// ticker_tx needs to be available from outside the loop and outside the thread, rx gets
		// cloned and moved inside the spawned thread to receive thread control messages
		let (ticker_tx, ticker_rx) = crossbeam_channel::unbounded();
		let logger_tx = self.logger_tx.clone();
		let url_string = self.url_string.clone();
		let db_tx = self.db_tx.clone();

		let mut should_restart = true;

		let myself:Actor = (*self).clone();

		// run() needs to be non-blocking so spawn a thread and give the main thread back to the caller
		std::thread::spawn(move ||{

			// Restart HTTP Connection
			loop {

				if should_restart {

					// This only really checks if it should run (a) on first execution or (b) when
					// a message is received that sets should_restart=true;

					println!("[ticker_controller] spawning ticker_loop_thread");

					// ***** Work Thread
					let myself2:Actor = myself.clone();
					std::thread::spawn(move ||

						// TODO: just pass the self, sheesh
						myself2.start_ticker_websocket(
							myself2.url_string.clone(),
							myself2.inbound_single_consumer.clone(),
							myself2.logger_tx.clone(),
							myself2.db_tx.clone(),
						)

					);

					println!("[ticker_controller] spawned ticker_loop_thread, waiting for message...");
				}


				// Comms: block until message received (requires an external message to restart the ticker)
				match single_consumer_clone.recv() {
					Ok(m) => {
						match m {
							MsgActor::Start => {
								println!("[listen] received Message::Start");
								// c_state.a_state.store(State::Started);
								should_restart = true;
							},
							MsgActor::Stop => {
								println!("[listen] received Message::Stop");
								// c_state.a_state.store(State::Stopped);
								let _ = ticker_tx.send(MsgActor::Stop);
								should_restart = false;
								return;
							},
							MsgActor::Pause => {
								println!("[listen] received Message::Pause");
								let _ = ticker_tx.send(MsgActor::Stop);
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
		println!("[run] '{}' listening", &name);

	}

	fn start_ticker_websocket(&self, url:String, rx: Receiver<MsgActor>, logger: Sender<MsgActor>, db: Sender<MsgActor>)->() {

		fn generate_subscribe_message() -> json::JsonValue {
			let subscribe_message = json::object!{
				"type": "subscribe",
				"product_ids": ["BTC-USD"],
				// "channels": ["heartbeat", "ticker"]
				"channels": ["ticker"]
			};
			subscribe_message.to_owned()
		}

		let is_pro = (*self).is_pro;
		match url::Url::parse(&url){
			Err(e) => println!("[ticker] parse(&url) failed: {}", &e),
			Ok(url) => {
				let mut do_restart = true;

				// Restart HTTP upgrade
				while do_restart {

					// Http Upgrade Request
					let websocket_result = tungstenite::connect(&url);

					match websocket_result {

						// websocket connected
						Ok((mut socket, _)) => {
							println!("[ticker] websocket connected");

							// Subscribe to Coinbase WebSocket
							let _ = socket.write_message(tungstenite::Message::Text(generate_subscribe_message().dump()));

							// WebSocket
							loop {

								// Read Websocket
								let ws_result = socket.read_message();
								match ws_result {

									// convert ticker json to object, send to database
									Ok(ws_mesg) => {
										match ws_mesg {
											tungstenite::Message::Text(t) => {

												// Parse websocket json "type" here: ticker, subscriptions,
												println!("[ticker_actor] ticker: {}", &t);
												// TODO: unwrap
												let json_val:serde_json::Value = serde_json::from_str(&t).unwrap();

												// use as_str() to remove the quotation marks
												// https://docs.serde.rs/serde_json/
												let ws_type = json_val["type"].as_str();
												println!("[ticker_actor] ws_type: {}", &ws_type.unwrap());

												match ws_type {
													Some("subscriptions") => {
														println!("[ticker_actor] json:subscriptions: {}", &t);

													},
													Some("ticker") => {
														// println!("[ticker_actor] json:ticker: {}", &t);
														let ticker2:Option<Ticker2> = serde_json::from_value(json_val).expect("[ticker_actor] json conversion to Ticker 2 didn't work"); // unwrap_or(None);
														if is_pro {
															// match convert_json_to_ticker(t){
															if let Some(t2) = ticker2 {
																	// let _ = db.send(MsgActor::DbInsertPro(ticker));
																	// println!("[ticker_actor] sending t2 to db/pro: {:?}", &t2);
																	let _ = db.send(MsgActor::SaveTickerPro(t2));
															};
														}else {
															// match convert_json_to_ticker(t){
															if let Some(t2) = ticker2 {
																// let _ = db.send(MsgActor::DbInsertPro(ticker));
																// println!("[ticker_actor] sending t2 to db/pro: {:?}", &t2);
																let _ = db.send(MsgActor::SaveTickerSand(t2));
															};
															// match convert_json_to_ticker(t){
															// 	None => {},
															// 	Some(ticker) => {
															// 		let _ = db.send(MsgActor::DbInsertSandbox(ticker));
															// 	}
															// }
														}
													},
													Some(unknown) => {
														println!("[ticker_actor] json:unknown: {}", &t);
													},
													None => println!("[ticker_actor] json conversion None"),
												}


												// let ticker_struct:Ticker2 = serde_json::from_str(&t).expect("[ticker_actor] conversion of json to Ticker2 didn't work");
												// println!("[ticker_actor] ticker_struct: {}", &ticker_struct);




											},
											_ => println!("[convert_json_to_ticker] unknown socket message or something not text"),
										}
									},
									Err(e) => {
										match e {
											tungstenite::error::Error::ConnectionClosed => {
												// https://docs.rs/tungstenite/0.11.1/tungstenite/error/enum.Error.html#variant.ConnectionClosed
												// TODO: stop the loop; attempt to reopen socket
												println!("[parse_incoming_socket_blocking] socket: Error::ConnectionClosed");

												// break from the websocket loop, fall back to the http loop
												do_restart = true;
												break;
											},
											_ => {
												println!("[parse_incoming_socket_blocking] socket read failed: {}",  &e);

												// break from the websocket loop, fall back to the http loop
												do_restart = true;
												break;
											}
										}
									},
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
						},
						Err(e) => {
							// TODO: loop and keep on trying
							println!("[ticker] websocket connection failed, trying in 2 seconds: {}", &e);
							std::thread::sleep(std::time::Duration::from_millis(2000));
							do_restart = true;
						},
					}
				}
			}
		}
	}



}


fn convert_json_to_ticker(t:String) ->Option<crate::actor_tools::Ticker>{

	if let Ok(j) = json::parse(&t) {


		use json::JsonValue;

		let result_opt = match &j["type"] {

			// json::Null => { println!("j[type] is null"); },
			// JsonValue::Array(s) => {println!("s:array: {:?}", &s)},
			// JsonValue::Boolean(s) => {println!("s:bool {}", &s)},
			// JsonValue::Number(s) => {/**/println!("s:nbr {}", &s)},
			// JsonValue::Object(s) => {println!("s:Obj: {:?}", &s)},
			// JsonValue::String(s) => {println!("s:String {}", &s)},
			JsonValue::Short(s) => {
				let ticker_opt = match s.to_string().as_str() {
					"heartbeat" => {
						println!("[convert_json_to_ticker] socket: coinbase heartbeat: {}", &j["type"]);
						None
					},
					"ticker" => {
						// println!("[convert_json_to_ticker] socket: ticker: {}", &j["type"]);
						let new_tick = crate::actor_tools::Ticker{
							//sequence:j["sequence"].to_string(),
							dtg:j["time"].to_string(),
							symbol:j["product_id"].to_string(),
							price:j["price"].to_string(),
							volume_24h:j["volume_24h"].to_string(),
						};
						// println!("[convert_json_to_ticker] ticker: {:?}", &new_tick);
						// db_write_ticker(&new_tick, &mut pg_client);
						Some(new_tick)
					},
					_ => {
						println!("[convert_json_to_ticker] socket: not ticker nor heartbeat");
						None
					},
				};
				ticker_opt

			},
			_ => {
				None
			}
		};
		result_opt
	} else {
		// json not parse-able
		println!("[parse_incoming_socket] incoming json on websocket not parse-able");
		None
	}
}




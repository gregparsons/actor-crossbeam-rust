

use crossbeam_utils::atomic::AtomicCell;
use crossbeam_channel::{Sender, Receiver};
use std::thread::{spawn};
use crate::actor_tools::{MsgActor, Ticker2};
use std::borrow::BorrowMut;


pub struct Actor {
	name:String,
	inbound_multi_producer:crossbeam_channel::Sender<MsgActor>,
	inbound_single_consumer:crossbeam_channel::Receiver<MsgActor>,
	parent_tx:Sender<MsgActor>,
	logger_tx:Sender<MsgActor>,

}

impl Actor {
	pub fn clone(&self) -> Actor {
		let copy = Actor {
			name:self.name.clone(),
			inbound_multi_producer:self.inbound_multi_producer.clone(),
			inbound_single_consumer:self.inbound_single_consumer.clone(),
			parent_tx:self.parent_tx.clone(),
			logger_tx:self.logger_tx.clone(),
		};
		copy
	}

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

	// Give the main thread a way to send messages TO me
	pub fn get_sender(&self) -> Sender<MsgActor>{
		self.inbound_multi_producer.clone()
	}

	pub fn run(&self){
		// this uses this to listen
		let single_consumer_clone = self.inbound_single_consumer.clone();
		let name = self.name.clone();
		let mut client_opt: Option<postgres::Client> = None;
		let mut is_connected = false;
		let myself:Actor = (*self).clone();




		// * DB ************************************************************************************

		// 1. Start DB connection, make it restartable on disconnect/fail
		// spawn(move || {
		// 	loop{
		//
		// 		// Loop
		// 		// 1. check if connected to postgresql
		// 		// 2. check if any messages from the outside
		//
		//
		//
		//
		//
		//
		//
		//
		//
		// 	}
		// });



		// 2. Listen...below...for insert commands
		// TODO: move the postgres client to the listener thread when it's live







		// * Listen Thread ************************************************************************************

		let parent_tx = self.parent_tx.clone();
		let rx = self.inbound_single_consumer.clone();
		spawn(move ||{
			loop {

				// Put DB reconnect work here?

				if !is_connected {
					// while not connected, attempt to reconnect
					let client = myself.db_connect();
					match client {
						None => {
							// try again,
							is_connected = false;
							println!("[DbActor] postgres not connected, sleeping 2 seconds");
							std::thread::sleep(std::time::Duration::from_secs(2));
						},
						Some(c) => {
							is_connected = true;
							println!("[DbActor] postgres connected");
							// c is now the postgres client
							client_opt = Some(c);

						},
					}
				}

				match rx.recv() {
					Ok(m) => {
						//println!("[listen] receive ok: {:?}", m);
						match m {
							MsgActor::Start => {
								println!("[DbActor] received start");
								// c_state.a_state.store(State::Started);
							},
							MsgActor::Stop => {
								println!("[DbActor] received stop");
								// c_state.a_state.store(State::Stopped);
								// TODO: Disconnect postgres?
								return;
							},
							MsgActor::DbInsertPro(ticker) =>{
								println!("[DbActor:DbInsertSandbox] ticker: {:?}", ticker);
								match &mut client_opt {
									None => {},
									Some(cli) => {
										let mut c2 = cli.borrow_mut();
										db_write_ticker(&ticker, c2, true);
									},
								}
							},
							MsgActor::DbInsertSandbox(ticker) =>{
								println!("[DbActor:DbInsertSandbox] ticker: {:?}", ticker);
								match &mut client_opt {
									None => {},
									Some(cli) => {
										let mut c2 = cli.borrow_mut();
										db_write_ticker(&ticker, c2, false);
									},
								}
							},
							MsgActor::SaveTickerPro(ticker) =>{
								// println!("[DbActor:SaveTickerPro] ticker: {:?}", ticker);
								match &mut client_opt {
									None => {},
									Some(cli) => {
										let mut c2 = cli.borrow_mut();
										db_write_ticker2(&ticker, c2, true);
									},
								}
							},
							MsgActor::SaveTickerSand(ticker) =>{
								// println!("[DbActor:SaveTickerSand] ticker: {:?}", ticker);
								match &mut client_opt {
									None => {},
									Some(cli) => {
										let mut c2 = cli.borrow_mut();
										db_write_ticker2(&ticker, c2, false);
									},
								}
							},
							_ => {}
						}
					},
					_ => { println!("[listen] receive error") }
				}
			}
		});
		println!("[run] '{}' listening", &name);
	}

	fn db_connect(&self) -> Option<postgres::Client> {

		// TODO: loop to keep trying indefinitely. if db doesn't connect, there's no point doing anything else. Block.

		println!("[DbActor::db_connect]");
		let database_url_result = std::env::var("COIN_DATABASE_URL");
		let client_opt = match database_url_result {
			Err(e) => {
				println!("[DbActor::db_connect] COIN_DATABASE_URL must be set: {}", &e);
				None
			},
			Ok(database_url) => {
				let client_result = postgres::Client::connect(&database_url, postgres::NoTls);
				match client_result {
					Err(e) 	=> {
						println!("[DbActor::db_connect] postgresql connection failed: {}", &e);
						None
					},
					Ok(client) 	=> {
						println!("[DbActor::db_connect] Connected to postgresql");
						Some(client)
					}
				}
			}
		};
		// return:
		client_opt
	}
}

fn db_write_ticker(t:&crate::actor_tools::Ticker, c:&mut postgres::Client, is_pro:bool){
	println!("[db_insert] t: {:?}", t);

	let sql_option = if is_pro {
		Some(format!("INSERT INTO coinbase_ticker(dtg, symbol, price, volume_24h) values ('{}','{}','{}','{}');", &t.dtg, &t.symbol, &t.price, &t.volume_24h))
	}else{
		Some(format!("INSERT INTO coinbase_ticker_sandbox(dtg, symbol, price, volume_24h) values ('{}','{}','{}','{}');", &t.dtg, &t.symbol, &t.price, &t.volume_24h))
	};

	match sql_option {
		None => {
			println!("[db_write_ticker] Because COINBASE_SANDBOX we don't know which SQL statement to use to store the ticker.");
		},
		Some(sql) => {
			// println!("[db_write_ticker] sql: {}",&sql);
			let result_vec_result = c.simple_query(&sql);
			match result_vec_result {
				Err(e) => {
					println!("[db_write_ticker] Postgres simple_query() failed: {}", &e);
				},
				Ok(result_vec) => {
					for i in result_vec {
						match i {
							postgres::SimpleQueryMessage::CommandComplete(x) =>
								println!("[db_write_ticker] Inserted: {} rows",x),
							_ => println!("[db_write_ticker] Something weird happened on db insert."),
						}
					}
				}
			}
		},
	}
}

fn db_write_ticker2(t:&Ticker2, c:&mut postgres::Client, is_pro:bool){
	println!("[db_write_ticker2] t: {:?}", t);

	/*
	pub ws_type: String,
	pub sequence:u64,
	pub product_id:String,
	pub price:f64,
	pub open24h:f64,
	pub volume_24h:f64,
	pub low_24h:f64,
	pub high_24h:f64,
	pub volume_30d:f64,
	pub best_bid:f64,
	pub best_ask:f64,
	pub side:String,
	pub time:String,
	pub trade_id:u64,
	pub last_size:f64,
	 */

	let sql_option = if is_pro {
		Some(format!("INSERT INTO t_cb_ticker(\
			dtg, \
			price, \
			volume_24h, \
			sequence, \
			product_id, \
			side, \
			open_24h, \
			low_24h, \
			high_24h, \
			volume_30d, \
			best_bid, \
			best_ask, \
			trade_id, \
			last_size \
		 ) values \
		('{}'::timestamp,'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::bigint,'{}'::varchar,'{}'::varchar,'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::bigint,'{}'::numeric(20,10));",
			&t.time,
			&t.price,
			&t.volume_24h,
			&t.sequence,
			&t.product_id,
			&t.side,
			&t.open_24h,
			&t.low_24h,
			&t.high_24h,
			&t.volume_30d,
			&t.best_bid,
			&t.best_ask,
			&t.trade_id,
			&t.last_size,
	/*

		create table if not exists t_cb_ticker_sand
		(
			id bigserial not null
				constraint t_cb_ticker_sand_pkey
					primary key,
			dtg timestamp,
			price numeric(20,10),
			volume_24h numeric(20,10),
			sequence bigint,
			product_id varchar,
			side varchar,
			open_24h  numeric(20,10),
			low_24h  numeric(20,10),
			high_24h  numeric(20,10),
			volume_30d  numeric(20,10),
			best_bid numeric(20,10),
			best_ask  numeric(20,10),
			trade_id bigint,
			last_size numeric(20,10)
		);
		alter table t_cb_ticker_sand owner to postgres;

	 */



		))
	}else{
		Some(format!("INSERT INTO t_cb_ticker_sand(\
			dtg, \
			price, \
			volume_24h, \
			sequence, \
			product_id, \
			side, \
			open_24h, \
			low_24h, \
			high_24h, \
			volume_30d, \
			best_bid, \
			best_ask, \
			trade_id, \
			last_size \
		 ) values \
		('{}'::timestamp,'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::bigint,'{}'::varchar,'{}'::varchar,'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::numeric(20,10),'{}'::bigint,'{}'::numeric(20,10));",
			 &t.time,
			 &t.price,
			 &t.volume_24h,
			 &t.sequence,
			 &t.product_id,
			 &t.side,
			 &t.open_24h,
			 &t.low_24h,
			 &t.high_24h,
			 &t.volume_30d,
			 &t.best_bid,
			 &t.best_ask,
			 &t.trade_id,
			 &t.last_size,
		))	};

	match sql_option {
		None => {
			// TODO: remove this Option. This is dumb.
			println!("[db_write_ticker] Because COINBASE_SANDBOX we don't know which SQL statement to use to store the ticker.");
		},
		Some(sql) => {
			// println!("[db_write_ticker] sql: {}",&sql);
			let result_vec_result = c.simple_query(&sql);
			match result_vec_result {
				Err(e) => {
					println!("[db_write_ticker2] Postgres simple_query() failed: {}", &e);
				},
				Ok(result_vec) => {
					for i in result_vec {
						match i {
							postgres::SimpleQueryMessage::CommandComplete(x) =>
								println!("[db_write_ticker2] Inserted: {} rows",x),
							_ => println!("[db_write_ticker2] Something weird happened on db insert."),
						}
					}
				}
			}
		},
	}
}


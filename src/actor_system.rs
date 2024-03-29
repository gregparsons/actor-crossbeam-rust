
/**

	A reference (crossbeam channel) to a logging actor is passed to the ping/pong acters
	so they can send messages to the logging actor directly. It somewhat tightens coupling
	but doesn't require the ping/pongs to be sent to this "system" first for relay to the
	logger. I like the idea of a central switchboard but it's twice the comms paths to keep track of.


	- TODO: actor system needs a way to listen from all children; needs to give a copy of its multi-producer
	end of its personal channel

	- TODO: some kind of addressing system so actors can send to each other directly and can reply
	directly to each other. This system probably needs a hash table of the multi-producer ends of each
	actor so any other actor can look them up, vice sending just to the parent expecting a relay.
	As an example, this "pinger" actor sends a ping and "ponger" wants to reply. Ideally they'd talk
	direct but in this case they're relaying via this system listener that just happens to know
	what they want to happen.

*/


use crossbeam_channel::{Sender, Receiver};
use std::thread::{spawn};
use crate::actor_tools::MsgActor;

pub fn start() -> () {

	// Main Comms channel: only this "actor" can receive but there can be many copies given out to subordinates
	let (main_sender, main_consumer):(Sender<MsgActor>, Receiver<MsgActor>) = crossbeam_channel::unbounded();

	// ********* Logging Consumer *********
	let logging_actor = crate::logging_actor::Actor::new("logger".to_string(), main_sender.clone());
	logging_actor.run();


	// ********* Database Consumer *********
	let db_actor = crate::db_actor::Actor::new(
		"db-actor".to_string(),
		main_sender.clone(),
		logging_actor.get_sender(),
	);
	db_actor.run();


	// TODO: move to ticker_actor
	// ********* Ticker Coinbase Consumer *********
	// let url_sandbox = "wss://ws-feed-public.sandbox.pro.coinbase.com".to_string(); //std::env::var("COINBASE_URL").expect("COINBASE_URL must be set");
	// let url_pro = "wss://ws-feed.pro.coinbase.com".to_string(); // std::env::var("COINBASE_URL").expect("COINBASE_URL must be set");

	if let Ok(sandbox_url) = std::env::var("CB_WS_URL_SANDBOX"){
		let ticker_actor = crate::ticker_actor::Actor::new(
			"ticker_sand".to_string(),
			sandbox_url,
			false,
			main_sender.clone(),
			logging_actor.get_sender(),
			db_actor.get_sender(),
		);
		ticker_actor.run();
	}else{
		println!("[ticker_actor.run] CB_WS_URL_SANDBOX env variable not found");
	};

	if let Ok(pro_url) = std::env::var("CB_WS_URL_PRO"){
		let ticker_actor = crate::ticker_actor::Actor::new(
			"ticker_sand".to_string(),
			pro_url,
			true,
			main_sender.clone(),
			logging_actor.get_sender(),
			db_actor.get_sender(),
		);
		ticker_actor.run();
	}else{
		println!("[ticker_actor.run] CB_WS_URL_PRO env variable not found");
	};




	// // start pong actor
	// let ponger_actor = crate::pinger_actor::Actor::new(
	// 	"ponger".to_string(),
	// 	main_sender.clone(),
	// 	logging_actor.get_sender()
	// );
	// ponger_actor.run(false);
	//
	// // start ping actor
	// let pinger_actor = crate::pinger_actor::Actor::new(
	// 	"pinger".to_string(),
	// 	main_sender.clone(),
	// 	logging_actor.get_sender()
	// );
	// pinger_actor.run(true);

	// Start Actor System listener thread
	// let ping = pinger_actor.get_sender();
	// let pong = ponger_actor.get_sender();
	// let logger = logging_actor.get_sender();

	let rx_system: Receiver<MsgActor> = main_consumer.clone();
	spawn(move || {
		loop {
			match rx_system.recv() {
				// Ok(MsgActor::Ping) => {
				// 	pong.send(MsgActor::Ping);
				// },
				// Ok(MsgActor::Pong) => {
				// 	ping.send(MsgActor::Pong);
				// },
				// Ok(MsgActor::LogPrint(msg)) => {
				// 	logger.send(MsgActor::LogPrint(msg));
				// },
				Ok(_msg) => {
					// logger.send(MsgActor::LogPrint(format!("[actor_system] misc message: {:?}", msg)));
				}
				_ => {
					println!("[ActorSystem] error receiving from switchboard");
				}
			}
		}
	});

	loop{};
	// sleep(Duration::from_secs(20));
	// // ponger_actor.get_sender().send(MsgActor::Stop);
	// // sleep(Duration::from_secs(5));
	// // pinger_actor.get_sender().send(MsgActor::Stop);
	// //logging_actor.get_sender().send(MsgActor::Stop);
	// ticker_actor.get_sender().send(MsgActor::Pause);
	// sleep(Duration::from_secs(10));
	// ticker_actor.get_sender().send(MsgActor::Start);
	// sleep(Duration::from_secs(15));
	// ticker_actor.get_sender().send(MsgActor::Stop);
	// sleep(Duration::from_secs(15));


}
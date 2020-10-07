
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


use std::time::{Duration, Instant};
use crossbeam_channel::{after, tick, Sender, Receiver, unbounded};
use std::thread::{spawn, sleep};
use crate::actor::{MsgActor};

pub fn start() -> () {

	// Main Comms channel: only this "actor" can receive but there can be many copies given out to subordinates
	let (main_sender, main_consumer):(Sender<MsgActor>, Receiver<MsgActor>) = unbounded();

	// start a logging actor
	let logging_actor = crate::logging_actor::Actor::new("logger".to_string(), main_sender.clone());
	logging_actor.run();

	// start a pong actor
	let ponger_actor = crate::pinger_actor::Actor::new(
		"ponger".to_string(),
		main_sender.clone(),
		logging_actor.get_sender()
	);
	ponger_actor.run(false);

	// start a pinger actor
	let pinger_actor = crate::pinger_actor::Actor::new(
		"pinger".to_string(),
		main_sender.clone(),
		logging_actor.get_sender()
	);
	pinger_actor.run(true);

	// Start Actor System listening inbox
	let rx_from_all_children: Receiver<MsgActor> = main_consumer.clone();

	let ping = pinger_actor.get_sender();
	let pong = ponger_actor.get_sender();
	let logger = logging_actor.get_sender();
	spawn(move || {
		loop {
			match rx_from_all_children.recv() {
				Ok(MsgActor::Ping) => {
					// tx_to_logging_actor.send(MessageRequest::LogPrint(format!("[actor_system] received: {:?}", MessageRequest::Ping)));
					pong.send(MsgActor::Ping);
				},
				Ok(MsgActor::Pong) => {
					// tx_to_logging_actor.send(MessageRequest::LogPrint(format!("[actor_system] received: {:?}", MessageRequest::Pong)));
					ping.send(MsgActor::Pong);
				},
				Ok(MsgActor::LogPrint(msg)) => {
					logger.send(MsgActor::LogPrint(msg));
				},
				Ok(response_message) => {
					logger.send(MsgActor::LogPrint(format!("[actor_system] misc message: {:?}", response_message)));
				},
				Err(e) => {
					println!("[main listener] error receiving from switchboard");
				}
			}
		}
	});


	// loop{};
	sleep(Duration::from_secs(10));
	ponger_actor.get_sender().send(MsgActor::Stop);
	sleep(Duration::from_secs(5));
	pinger_actor.get_sender().send(MsgActor::Stop);
	logging_actor.get_sender().send(MsgActor::Stop);

}
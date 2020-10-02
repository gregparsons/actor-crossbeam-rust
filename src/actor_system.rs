
/**

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
use crate::actor::{MessageRequest};

pub fn start() -> () {

	// Main Comms channel: only this "actor" can receive but there can be many copies given out to subordinates
	let (main_sender, main_consumer):(Sender<MessageRequest>, Receiver<MessageRequest>) = unbounded();

	// start a logging actor
	let logging_actor = crate::logging_actor::Actor::new();
	// let tx_to_logging_actor = logging_actor.get_sender();
	// let rx_from_logging_actor = logging_actor.get_receiver();
	logging_actor.run();

	// start a pong actor
	let ponger_actor = crate::pinger_actor::Actor::new(main_sender.clone());
	// let tx_to_ponger_actor = pinger_actor.get_sender();
	// let rx_from_ponger_actor = pinger_actor.get_receiver();
	ponger_actor.run(false);

	// start a pinger actor
	let pinger_actor = crate::pinger_actor::Actor::new(main_sender.clone());
	// let tx_to_pinger_actor = pinger_actor.get_sender();
	// let rx_from_pinger_actor = pinger_actor.get_receiver();
	pinger_actor.run(true);


	// Start Actor System listening inbox
	// TODO: need to be able to consumer many producers here, not just one
	// TODO: each new child actor needs to get a copy of actor_system's transmitter
	let rx_from_all_children: Receiver<MessageRequest> = main_consumer.clone();
	loop {
		match rx_from_all_children.recv() {
			Ok(MessageRequest::Ping) => {
				// tx_to_logging_actor.send(MessageRequest::LogPrint(format!("[actor_system] received: {:?}", MessageRequest::Ping)));
				ponger_actor.get_sender().send(MessageRequest::Ping);
			},
			Ok(MessageRequest::Pong) => {
				// tx_to_logging_actor.send(MessageRequest::LogPrint(format!("[actor_system] received: {:?}", MessageRequest::Pong)));
				pinger_actor.get_sender().send(MessageRequest::Pong);
			},
			Ok(MessageRequest::LogPrint(msg)) => {
				logging_actor.get_sender().send(MessageRequest::LogPrint(msg));
			},
			Ok(response_message) => {
				logging_actor.get_sender().send(MessageRequest::LogPrint(format!("[actor_system] misc message: {:?}", response_message)));
			},
			Err(e) => {
				println!("[main listener] error receiving from switchboard");
			}
		}
	}
}
mod db_actor;
mod actor_tools;
mod ticker_actor;
/**

	A rudimentary actor system in Rust. Very rudimentary.

	Use: crossbeam to effect minimal actors as threads

	Actors are long-running threads and communicate over Crossbeam unbounded channels.

	The idea is to ingest a streaming data feed say from a websocket then persist the data
	to a data store. Both the websocket reader and the database need to not crash and if they
	do, not bring down the other. They also need to not block each other when doing their work.
	Socket reader will get data in, send a message, database thread will hear that message and
	store the data in the database on its own time.

	Another actor should monitor both the database and websocket to ensure they're up. If they're
	not it should send a message to the system to restart either. There will be another thread to
	perform actions based on data analysis, then make another rest call based on that analysis.

	Usually the network/websocket would die for some reason. Don' want it to bring down the other
	actors, in the spirit of the actor model it should fail safely and restart correctly, hopefully
	automatically.

*/

mod pinger_actor;
mod actor_system;
mod actor;
mod logging_actor;

fn main() {
	dotenv::dotenv().ok();
	actor_system::start();
}


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

// https://docs.rs/crossbeam/0.7.3/crossbeam/channel/index.html
// https://docs.rs/crossbeam-channel/0.4.4/crossbeam_channel/

fn main() {
	/*
	/**************************************************/
	println!("Bounded demo: ");
	let (s1, r1) = bounded(0);
	let (s2, r2) = (s1.clone(), r1.clone());

	// Spawn a thread that receives a message and then sends one.
	std::thread::spawn(move || {
		let rcvd = r2.recv().unwrap();
		println!("received: {}", rcvd);
		s2.send(2).unwrap();
	});

	// Send a message and then receive one.
	s1.send(1).unwrap();
	let rcvd = r1.recv().unwrap();
	println!("received: {}", rcvd);



	/**************************************************/
	println!("Unbounded demo: ");

	// Create a channel of unbounded capacity.
	let (s, r) = unbounded();

	// Receive the message from the channel.
	std::thread::spawn(move||{
		println!("spawned thread. assert...");
		// blocks on recv()
		assert_eq!(r.recv(), Ok("Hello, world!"));
	});

	std::thread::sleep(std::time::Duration::from_millis(2000));

	// Send a message into the channel.
	s.send("Hello, world!").unwrap();


	/**************************************************/
	println!("Ticker demo: ");

	// Ticker
	let start = Instant::now();
	let ticker:crossbeam_channel::Receiver<Instant> = tick(Duration::from_millis(200));
	let timeout = after(Duration::from_secs(10));

	// move forces the thread closure to own ticker and timeout
	spawn(move || {
		loop {
			crossbeam_channel::select! {
				recv(ticker) -> _ => println!("elapsed: {:?}", start.elapsed()),
				recv(timeout) -> _ => break,
			}
		};
	});

	*/

	actor_system::start();

}


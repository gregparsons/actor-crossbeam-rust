
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MsgActor {
	Start,
	Stop,
	Pause,
	Ping,
	Pong,
	LogPrint(String),
	DbInsertPro(Ticker),
	DbInsertSandbox(Ticker),
	SaveTickerPro(Ticker2),
	SaveTickerSand(Ticker2),


}

#[derive(Debug, Copy, Clone)]
pub enum State {
	Started, Stopped
}

pub struct ActorState {
	pub a_state:crossbeam_utils::atomic::AtomicCell<State>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Ticker {
	// Received: {"type":"ticker","sequence":194050273,"product_id":"BTC-USD","price":"10437.79","open_24h":"10370.9","volume_24h":"908177.98995845","low_24h":"10279.99","high_24h":"10611.83","volume_30d":"2541970.84037783","best_bid":"10437.79","best_ask":"10437.81","side":"sell","time":"2020-09-13T01:07:11.688038Z","trade_id":15646755,"last_size":"0.00095805"}
	//type:String,
	pub dtg:String,
	// sequence:String,
	pub symbol:String,
	pub price:String,
	pub volume_24h:String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Ticker2 {
	// Received: {"type":"ticker","sequence":194050273,"product_id":"BTC-USD","price":"10437.79","open_24h":"10370.9",
	// "volume_24h":"908177.98995845","low_24h":"10279.99","high_24h":"10611.83","volume_30d":"2541970.84037783",
	// "best_bid":"10437.79","best_ask":"10437.81","side":"sell","time":"2020-09-13T01:07:11.688038Z",
	// "trade_id":15646755,"last_size":"0.00095805"}
	#[serde(rename = "type")]
	pub ws_type: String,
	pub sequence:u64,
	pub product_id:String,
	pub price:String,
	pub open_24h:String,
	pub volume_24h:String,
	pub low_24h:String,
	pub high_24h:String,
	pub volume_30d:String,
	pub best_bid:String,
	pub best_ask:String,
	pub side:String,
	pub time:String,
	pub trade_id:u64,
	pub last_size:String,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct Ticker3 {
	// Received: {"type":"ticker","sequence":194050273,"product_id":"BTC-USD","price":"10437.79","open_24h":"10370.9",
	// "volume_24h":"908177.98995845","low_24h":"10279.99","high_24h":"10611.83","volume_30d":"2541970.84037783",
	// "best_bid":"10437.79","best_ask":"10437.81","side":"sell","time":"2020-09-13T01:07:11.688038Z",
	// "trade_id":15646755,"last_size":"0.00095805"}
	#[serde(rename = "type")]
	pub ws_type: String,
	pub sequence:u64,
	pub product_id:String,
	pub price:String,
	pub open_24h:String,
	pub volume_24h:String,
	pub low_24h:String,
	pub high_24h:String,
	pub volume_30d:String,
	pub best_bid:String,
	pub best_ask:String,
	pub side:String,
	pub time:String,
	pub trade_id:u64,
	pub last_size:String,

}



// use json::JsonValue;
// #[derive(Debug, Clone)]
// pub enum JsonValue {
// 	Null,
// 	Short(json::JsonValue::short::Short),
// 	String(String),
// 	Number(json::JsonValue::number::Number),
// 	Boolean(bool),
// 	Object(json::JsonValue::object::Object),
// 	Array(Vec<JsonValue>),
// }
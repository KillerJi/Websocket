extern crate futures;
extern crate tokio;
extern crate websocket;

use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use std::io::stdin;
use std::thread;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};

const CONNECTION: &'static str = "wss://echo.websocket.org";//wss://stream.binance.com:9443/ws
//{ "method": "SUBSCRIBE", "params": [ "btcusdt@aggTrade", "btcusdt@depth" ], "id": 1 }
fn main() {
	println!("Connecting to {}", CONNECTION);
	let mut runtime = tokio::runtime::current_thread::Builder::new()
		.build()
		.unwrap();
	let mut testws = format!("{{ \"method\": \"SUBSCRIBE\", \"params\": [ \"btcusdt@aggTrade\", \"btcusdt@depth\" ], \"id\": 1 }}");
	let a = r#"{ "method": "SUBSCRIBE", "params": [ "btcusdt@aggTrade", "btcusdt@depth" ], "id": 1 }"#;
	println!("{:?}",a);
	println!("{:?}",testws);
	// standard in isn't supported in mio yet, so we use a thread
	// see https://github.com/carllerche/mio/issues/321
	let (usr_msg, stdin_ch) = mpsc::channel(0);
	thread::spawn(move|| {
		let mut input = String::new();
		let mut stdin_sink = usr_msg.wait();
		loop {
			input.clear();
			stdin().read_line(&mut input).unwrap();
			input = testws.clone();
			let trimmed = input.trim();

			let (close, mut msg) = match trimmed {
				"/close" => (true, OwnedMessage::Close(None)),
				"/ping" => (false, OwnedMessage::Ping(b"PING".to_vec())),
				_ => (false, OwnedMessage::Text(trimmed.to_string())),
			};
			msg = OwnedMessage::Text(a.to_string().clone());
			stdin_sink
				.send(msg)
				.expect("Sending message across stdin channel.");

			if close {
				break;
			}
		}
	});

	let runner = ClientBuilder::new(CONNECTION)
		.unwrap()
		.async_connect_secure(None)
		.and_then(|(duplex, _)| {
			let (sink, stream) = duplex.split();
			stream
				.filter_map(|message| {
					println!("Received Message: {:?}", message);
					match message {
						OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
						OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
						_ => None,
					}
				})
				.select(stdin_ch.map_err(|_| WebSocketError::NoDataAvailable))
				.forward(sink)
		});
		println!("1");
	let _ = runtime.block_on(runner).unwrap();
	println!("1");
}

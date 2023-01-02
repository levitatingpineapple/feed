use std::{time::{Duration, Instant}};
use actix_ws::Message;
use futures_util::{future::{self, Either}, StreamExt as _,};
use matrix_sdk::{ruma::{events::room::message::{MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent}}, room::{Room, Joined}, Client, event_handler::Ctx};
use tokio::{pin, time::interval};

use crate::matrix::*;

pub async fn handler(
	client: Client,
	joined: Joined, 
	mut session: actix_ws::Session,
	mut message_stream: actix_ws::MessageStream,
) {
	client.add_event_handler_context(session.clone());
	let handle = joined.add_event_handler(|
		event: OriginalSyncRoomMessageEvent, 
		_room: Room, 
		session: Ctx<actix_ws::Session>
	| async move {
		println!("{}", event.sender);
		if let MessageType::Text(text) = event.content.msgtype {
			let mut session = session.clone();
			session.text(
				format!(
					"<p class=\"{}\">{}</p>", 
					if event.sender == "@bot:n0g.rip" { "me" } else { "you" }, 
					text.body
				)
			).await.unwrap();
		};
	});
	
	for event in messages(&client, Some(joined.clone())).await.iter().rev() {
		if let MessageType::Text(text) = &event.content.msgtype {
			session.text(
				format!(
					"<p class=\"{}\">{}</p>", 
					if event.sender == "@bot:n0g.rip" { "me" } else { "you" }, 
					text.body
				)
			).await.unwrap();
		};
	}
	session.text(":loaded").await.unwrap();

	// Web socket
	let mut last_heartbeat = Instant::now();
	let mut interval = interval(Duration::from_secs(5));
	let reason = loop {
		let tick = interval.tick();
		pin!(tick);
		match future::select(message_stream.next(), tick).await {
			Either::Left((Some(Ok(msg)), _)) => {
				println!("msg: {:?}", msg);
				match msg {
					Message::Text(text) => {
						joined.send(RoomMessageEventContent::text_plain(text), None).await.unwrap();
					}
					Message::Binary(bin) => {
						session.binary(bin).await.unwrap();
					}
					Message::Close(reason) => {
						break reason;
					}
					Message::Ping(bytes) => {
						last_heartbeat = Instant::now();
						let _ = session.pong(&bytes).await;
					}
					Message::Pong(_) => {
						last_heartbeat = Instant::now();
					}
					Message::Continuation(_) => { }
					Message::Nop => { }
				};
			}
			Either::Left((Some(Err(err)), _)) => {
				println!("{}", err);
				break None;
			}
			Either::Left((None, _)) => {
				break None;
			}
			Either::Right((_inst, _)) => {
				if Instant::now().duration_since(last_heartbeat) > Duration::from_secs(10) {
					println!("timeout");
					break None;
				}
				let _ = session.ping(b"").await;
			}
		}
	};
	client.remove_event_handler(handle);
	let _ = session.close(reason).await;
	println!("disconnected");
}
use matrix_sdk::{config::SyncSettings, ruma::{*, events::{*,room::message::RoomMessageEventContent}}, Client, Room};

pub type Message = OriginalMessageLikeEvent<RoomMessageEventContent>;

pub async fn client(id: &str, password: &str) -> Client {
	let user_id = UserId::parse(id)
		.expect("Invalid user ID!");
	let client = Client::builder()
		.server_name(user_id.server_name())
		.build()
		.await.expect("Invalid server name!");
	client
		.matrix_auth()
		.login_username(user_id, password)
		.send()
		.await.expect("Login failed!");
	let client_sync = client.clone();
	tokio::spawn(async move { 
		client_sync
			.sync(SyncSettings::default())
			.await.expect("Sync failed!");
	});
	client
}

pub async fn name(room: &Room) -> Option<String> {
	room.display_name().await.ok()
		.map(|s| s.to_string())
}

pub async fn messages(room: &Room) -> Vec<Message> {
	let mut options = matrix_sdk::room::MessagesOptions::backward();
	options.limit = uint!(32);
	room
		.messages(options)
		.await.unwrap()
		.chunk.iter()
		.filter_map(|timeline| {
			if let Ok(
				AnyTimelineEvent::MessageLike(
					AnyMessageLikeEvent::RoomMessage(
						MessageLikeEvent::Original(content)
					)
				)
			) = timeline.event.deserialize() {
				Some(content)
			} else { None }
		})
		.collect()
}

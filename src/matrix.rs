use matrix_sdk::{ruma::{*, events::{*,room::{message::{RoomMessageEventContent}}}}, Client, config::SyncSettings};

type Message = OriginalMessageLikeEvent<RoomMessageEventContent>;

pub const FEED: &str = "!bUtdRxQiBPeYOa3Z:n0g.rip";
pub const LOBBY: &str = "!OV5DspdU2TK4dTAq:n0g.rip";

pub async fn client() -> matrix_sdk::Client {
	let bot = user_id!("@bot:n0g.rip");
	let client = matrix_sdk::Client::builder()
		.server_name(bot.server_name())
		.build()
		.await.unwrap();
	client.login_username(bot, "sorzon-korqi7-sekWug").send().await.unwrap();
	client.sync_once(SyncSettings::default()).await.unwrap();
	println!("Client connected!");
	let client_sync = client.clone();
	tokio::spawn(async move { 
		client_sync.sync(SyncSettings::default()).await.unwrap();
	});
	client
}

pub async fn messages(client: &Client, room: &str) -> Vec<Message> {
	let mut options = matrix_sdk::room::MessagesOptions::backward();
	options.limit = uint!(100);
	client.get_joined_room(<&RoomId>::try_from(room).unwrap()).unwrap()
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
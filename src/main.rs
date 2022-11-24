use std::io::Write;
use js_int::uint;
use matrix_sdk::{
	Client, 
	config::SyncSettings,
	ruma::{
		user_id, 
		room_id, 
		events::{
			room::{
				message::MessageType,
				MediaSource
			}, 
			MessageLikeEvent, 
			AnyTimelineEvent, 
			AnyMessageLikeEvent
		},
	}, 
	room::MessagesOptions, 
	media::{MediaFormat, MediaRequest}, 
	Media,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let bot = user_id!("@bot:n0g.rip");
	let client = Client::builder()
		.server_name(bot.server_name())
		.build()
		.await?;
	client.login_username(bot, "sorzon-korqi7-sekWug")
		.send()
		.await?;
	client.sync_once(SyncSettings::default()).await?;
	if let Some(joined_room) = client.get_joined_room(room_id!("!xLb6sbIQiWRiRuXt:n0g.rip")) {
		let mut options = MessagesOptions::backward();
		options.limit = uint!(1000);
		let messages = joined_room.messages(options).await?;
		for timeline_event in messages.chunk.iter() {
			if let Ok(
				AnyTimelineEvent::MessageLike(
					AnyMessageLikeEvent::RoomMessage(
						MessageLikeEvent::Original(message)
					)
				)
			) = timeline_event.event.deserialize() {
				match message.content.msgtype {
					MessageType::Audio(audio) => {
						write_media_content(
							&audio.source, 
							&audio.body, 
							client.media()
						).await;
						println!("{}", audio .body);
					},
					MessageType::Image(image) => {
						write_media_content(
							&image.source, 
							&image.body, 
							client.media()
						).await;
						println!("{}", image.body);
					},
					MessageType::Text(text) => {
						println!("{}", text.body);
					}
					_ => {}
				}
			}
		}
	} else { panic!("Room not found!"); }
	anyhow::Ok(())
}

async fn write_media_content(source: &MediaSource, file_name: &String, media: Media) {
	if let Ok(data) = media.get_media_content(
		&MediaRequest {
			source: source.clone(),
			format: MediaFormat::File
		},
		false
	).await {
		let mut file = std::fs::File::create(format!("./static/{}", file_name))
			.expect("Unable to create file");
		file.write_all(&data)
			.expect("Unable to write data");
	}
}
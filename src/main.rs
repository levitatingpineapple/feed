use std::io::Write;

use matrix_sdk::{
	Client, 
	config::SyncSettings,
	media::{
		MediaRequest,
		MediaFormat
	},
	ruma::{
		user_id, 
		events::room::{
			message::{
				SyncRoomMessageEvent, 
				MessageType
			}, MediaSource
		},
	},
	room::Room, Media,
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
	client.add_event_handler(on_room_message);
	client.sync(SyncSettings::default())
		.await?;
	Ok(())
}

async fn on_room_message(ev: SyncRoomMessageEvent, room: Room, client: Client) {
	if room.room_id().as_str() == "!xLb6sbIQiWRiRuXt:n0g.rip" {
		match &ev.as_original().unwrap().content.msgtype {
			MessageType::Audio(audio) => {
				write_media_content(
					&audio.source, 
					&audio.body, 
					client.media()
				).await;
			},
			MessageType::Image(image) => {
				write_media_content(
					&image.source, 
					&image.body, 
					client.media()
				).await;
			},
			MessageType::Text(text) => {
				println!("{}", text.body);
			}
			_ => {}
		}
	}
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
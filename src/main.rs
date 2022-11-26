use chrono::*;
use std::io::Write;
use matrix_sdk::{
	ruma::{*,events::{*,room::{message::MessageType, MediaSource}}}, 
	media::MediaEventContent
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let bot = user_id!("@bot:n0g.rip");
	let client = matrix_sdk::Client::builder()
		.server_name(bot.server_name())
		.build()
		.await?;
	client.login_username(bot, "sorzon-korqi7-sekWug")
		.send()
		.await?;
	client.sync_once(matrix_sdk::config::SyncSettings::default()).await?;
	if let Some(joined_room) = client.get_joined_room(room_id!("!xLb6sbIQiWRiRuXt:n0g.rip")) {
		let mut options = matrix_sdk::room::MessagesOptions::backward();
		options.limit = js_int::uint!(10000);
		let messages = joined_room.messages(options).await?;
		let mut buffer = Vec::new();
		write_leading(&mut buffer);
		for timeline_event in messages.chunk.iter() {
			if let Ok(
				AnyTimelineEvent::MessageLike(
					AnyMessageLikeEvent::RoomMessage(
						MessageLikeEvent::Original(message)
					)
				)
			) = timeline_event.event.deserialize() {
				write_message(message.content.msgtype, &mut buffer);
				write_time(message.origin_server_ts, &mut buffer);
			}
		}
		write_trailing(&mut buffer);
		std::fs::write("./feed.html", buffer).expect("Unable to write file");
	} else { panic!("Room not found!"); }
	client.logout().await?;
	anyhow::Ok(())
}

fn write_leading(buffer: &mut Vec<u8>) {
	write!(buffer,
r#"
<!DOCTYPE html>
<html>
<head>
	<title>Feed</title>
	<meta charset="UTF-8">
	<meta name="viewport" content="initial-scale=1">
	<link rel="stylesheet" href="feed.css">
</head>
	<body>
"#
	).unwrap();
}

fn write_time(ms: MilliSecondsSinceUnixEpoch, buffer: &mut Vec<u8>) {
	let system_time = ms.to_system_time().unwrap();
	let date = DateTime::<Local>::from(system_time);
	write!(
		buffer, 
r#"		<time>{}</time>
"#, 
		date.format("%b %d, %H:%M").to_string()
	).unwrap();
}

fn write_message(message_type: MessageType, buffer: &mut Vec<u8>) {
	let download_path: &str = "https://n0g.rip/_matrix/media/r0/download/n0g.rip/";
	match message_type {
		MessageType::Audio(audio) => {
			if let MediaSource::Plain(uri) = audio.source {
				write!(buffer,
r#"		<audio controls>
			<source src="{}{}" type="{}">
		</audio>
"#,
					download_path,
					uri.media_id().unwrap(), 
					audio.info.unwrap().mimetype.unwrap()
				).unwrap();
			}
		}
		MessageType::Image(image) => {
			if let MediaSource::Plain(uri) = image.source {
				write!(buffer,
r#"		<img src="{}{}" type="{}">
"#,
					download_path,
					uri.media_id().unwrap(),
					image.info.unwrap().mimetype.unwrap()
				).unwrap();
			}
		}
		MessageType::Text(text) => {
			write!(buffer,
r#"		<p>{}</p>
"#,
				text.body.replace("\n", "<br>"),
			).unwrap();
		}
		MessageType::Video(video) => {
			if let Some(MediaSource::Plain(thumbnail_source)) = video.thumbnail_source() {
				if let MediaSource::Plain(uri) = video.source {
					write!(buffer,
r#"		<video controls poster="{}{}">
			<source src="{}{}" type="{}">
		</video>
"#,
						download_path,
						thumbnail_source.media_id().unwrap(),
						download_path,
						uri.media_id().unwrap(), 
						video.info.unwrap().mimetype.unwrap()
					).unwrap();
				}
			}
		}
		_ => { }
	}
}

fn write_trailing(buffer: &mut Vec<u8>) {
	write!(buffer,
r#"	</body>
</html>
"#
	).unwrap();
}
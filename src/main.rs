use matrix_sdk::{
	ruma::{
		*, 
		events::{
			*,
			room::{
				message::MessageType, 
				MediaSource
			}, 
		}
	}, 
	media::MediaEventContent
};
use std::io::Write;

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
				println!("{:?}", message.content.body());
				write_message(message.content.msgtype, &mut buffer);
			}
		}
		write_trailing(&mut buffer);
		std::fs::write("./feed.html", buffer).expect("Unable to write file");
	} else { panic!("Room not found!"); }
	anyhow::Ok(())
}

fn write_leading(buffer: &mut Vec<u8>) {
	write!(buffer,
r#"
<!DOCTYPE html>
<html>
<head>
	<title>Feed</title>
	<meta name="viewport" content="initial-scale=1">
	<style type="text/css">
		body {{
			color: white;
			background-color: #121212;
			font-family: sans-serif;
		}}
		video, audio, p, img {{
			display: block;
			margin: auto;
			width: min(640px, 92vw);
			margin-top: 32px;
			margin-bottom: 32px;
		}}
		video, img, p {{
			border-radius: 16px;
		}}
		p {{
			width: min(608px, calc(92vw - 32px));
			background-color: #2C2C2C;
			padding: 16px;
			line-height: 1.4em;
		}}
	</style>
</head>
	<body>
"#
	).unwrap();
}

fn write_message(message_type: MessageType, buffer: &mut Vec<u8>) {
	match message_type {
		MessageType::Audio(audio) => {
			if let MediaSource::Plain(uri) = audio.source {
				write!(buffer,
r#"		<audio controls>
			<source src="https://n0g.rip/_matrix/media/r0/download/n0g.rip/{}" type="{}">
		</audio>
"#, 
					uri.media_id().unwrap(), 
					audio.info.unwrap().mimetype.unwrap()
				).unwrap();
			}
		}
		MessageType::Image(image) => {
			if let MediaSource::Plain(uri) = image.source {
				write!(buffer,
r#"		<img src="https://n0g.rip/_matrix/media/r0/download/n0g.rip/{}" type="{}">
"#,
					uri.media_id().unwrap(),
					image.info.unwrap().mimetype.unwrap()
				).unwrap();
			}
		}
		MessageType::Text(text) => {
			write!(buffer,
r#"		<p>{}</p>
"#,
				text.body,
			).unwrap();
		}
		MessageType::Video(video) => {
			if let Some(MediaSource::Plain(thumbnail_source)) = video.thumbnail_source() {
				if let MediaSource::Plain(uri) = video.source {
					write!(buffer,
r#"		<video controls poster="https://n0g.rip/_matrix/media/r0/download/n0g.rip/{}">
			<source src="https://n0g.rip/_matrix/media/r0/download/n0g.rip/{}" type="{}">
		</video>
"#, 				
						thumbnail_source.media_id().unwrap(),
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
r#"
	</body>
</html>
"#
	).unwrap();
}
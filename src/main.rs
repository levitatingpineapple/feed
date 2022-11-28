use chrono::*;
use actix_web::*;
use std::io::Write;
use matrix_sdk::{
	ruma::{*,events::{*,room::{message::{MessageType, RoomMessageEventContent}, MediaSource}}}, 
	media::MediaEventContent
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	HttpServer::new(|| App::new()
		.route("/feed", web::get().to(root))
		.route("/feed/style", web::get().to(style))
		.route("/rss", web::get().to(rss))
	).bind(("127.0.0.1", 5002))?.run().await
}

async fn root(_req: HttpRequest) -> impl Responder {
	if let Some(messages) = messages().await {
		let mut buffer = Vec::new();
		write_leading(&mut buffer);
		for message in messages {
			write_message(&message.content.msgtype, &mut buffer);
			write_time(message.origin_server_ts, &mut buffer);
		}
		write_trailing(&mut buffer);
		if let Ok(string) = String::from_utf8(buffer) { 
			return HttpResponse::Ok().body(string);
		}
	}
	HttpResponse::Ok().body("Error")
}

async fn style(_req: HttpRequest) -> impl Responder {
	HttpResponse::Ok().body(include_str!("./style.css"))
}

async fn rss(_req: HttpRequest) -> HttpResponse {
	HttpResponse::Ok()
		.content_type(http::header::ContentType::xml())
		.body(include_str!("./test.rss").to_string())
}

async fn messages() -> Option<Vec<OriginalMessageLikeEvent<RoomMessageEventContent>>> {
	let bot = user_id!("@bot:n0g.rip");
	let client = matrix_sdk::Client::builder()
		.server_name(bot.server_name()).build().await.unwrap();
	client.login_username(bot, "sorzon-korqi7-sekWug").send().await.unwrap();
	client.sync_once(matrix_sdk::config::SyncSettings::default()).await.unwrap();
	if let Some(joined_room) = client.get_joined_room(room_id!("!xLb6sbIQiWRiRuXt:n0g.rip")) {
		let mut options = matrix_sdk::room::MessagesOptions::backward();
		options.limit = uint!(100);
		return Some(
			joined_room
				.messages(options)
				.await.unwrap()
				.chunk.iter()
				.filter_map(|event| {
					if let Ok(
						AnyTimelineEvent::MessageLike(
							AnyMessageLikeEvent::RoomMessage(
								MessageLikeEvent::Original(content)
							)
						)
					) = event.event.deserialize() {
						Some(content)
					} else { None }
				})
				.collect()
		)
	}
	client.logout().await.unwrap();
	None
}

fn write_leading(buffer: &mut Vec<u8>) {
	write!(buffer,
r#"<!DOCTYPE html>
<html>
<head>
	<title>Feed</title>
	<meta charset="UTF-8">
	<link rel="icon" type="image/png" href="../favicon.png">
	<meta name="viewport" content="initial-scale=1">
	<link rel="stylesheet" href="/feed/style">
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

fn write_message(message_type: &MessageType, buffer: &mut Vec<u8>) {
	let download_path: &str = "https://n0g.rip/_matrix/media/r0/download/n0g.rip/";
	match message_type {
		MessageType::Audio(audio) => {
			if let MediaSource::Plain(uri) = &audio.source {
				write!(buffer,
r#"		<audio controls>
			<source src="{}{}" type="{}">
		</audio>
"#,
					download_path,
					uri.media_id().unwrap(), 
					audio.clone().info.unwrap().mimetype.unwrap()
				).unwrap();
			}
		}
		MessageType::Image(image) => {
			if let MediaSource::Plain(uri) = &image.source {
				write!(buffer,
r#"		<img src="{}{}" type="{}">
"#,
					download_path,
					uri.media_id().unwrap(),
					image.clone().info.unwrap().mimetype.unwrap()
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
				if let MediaSource::Plain(uri) = &video.source {
					write!(buffer,
r#"		<video controls poster="{}{}">
			<source src="{}{}" type="{}">
		</video>
"#,
						download_path,
						thumbnail_source.media_id().unwrap(),
						download_path,
						uri.media_id().unwrap(), 
						video.clone().info.unwrap().mimetype.unwrap()
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

use rss::*;
use chrono::*;
use matrix_sdk::{ruma::{*, events::{*,room::{message::{MessageType, RoomMessageEventContent}, MediaSource}}}};
type Message = OriginalMessageLikeEvent<RoomMessageEventContent>;

async fn messages() -> Vec<Message> {
	let bot = user_id!("@bot:n0g.rip");
	let client = matrix_sdk::Client::builder()
		.server_name(bot.server_name())
		.build()
		.await.unwrap();
	client.login_username(bot, "sorzon-korqi7-sekWug").send().await.unwrap();
	client.sync_once(matrix_sdk::config::SyncSettings::default()).await.unwrap();
	let mut options = matrix_sdk::room::MessagesOptions::backward();
	options.limit = uint!(100);
	client.get_joined_room(room_id!("!xLb6sbIQiWRiRuXt:n0g.rip")).unwrap()
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

pub async fn http_body() -> String {
	messages().await.iter().map(|m| { 
		m.to_html_string()
	})
	.collect::<String>()
}

pub async fn rss() -> String {
	ChannelBuilder::default()
		.title("n0g.rip".to_string())
		.link("http://n0g.rip".to_string())
		.description("Feed".to_string())
		.items(
			messages().await.iter().map(|m| {
				let title = Some(match m.content.msgtype {
					MessageType::Audio(_) => "Audio",
					MessageType::Image(_) => "Image",
					MessageType::Text(_) => "Text",
					MessageType::Video(_) => "Video",
					_ => "Message",
				}.to_string());
				let guid = Some(Guid { 
					value: m.origin_server_ts.get().to_string(), 
					permalink: false
				});
				let content = Some(m.content.clone().msgtype.to_html_string());
				let pub_date = Some(
					DateTime::<Local>::from(m.origin_server_ts
						.to_system_time()
						.unwrap()
					).to_rfc2822()
				);
				ItemBuilder::default()
					.title(title)
					.guid(guid)
					.content(content)
					.pub_date(pub_date)
					.build()
			})
			.collect::<Vec<Item>>()
		)
		.build()
		.to_string()
}

trait ToHtmlString { 
	fn to_html_string(&self) -> String;
}

impl ToHtmlString for MilliSecondsSinceUnixEpoch {
	fn to_html_string(&self) -> String {
		format!(
			"<time>{}</time>", 
			DateTime::<Local>::from(self.to_system_time().unwrap())
				.format("%b %d, %H:%M")
				.to_string()
		)
	}
}

impl ToHtmlString for MessageType {
	fn to_html_string(&self) -> String {
		let download_path: &str = "https://n0g.rip/_matrix/media/r0/download/n0g.rip/";
		match self {
			MessageType::Audio(audio) => 
				if let MediaSource::Plain(uri) = &audio.source {
					format!(	
						"<audio controls><source src=\"{}{}\" type=\"{}\"></audio>",
						download_path,
						uri.media_id().unwrap(), 
						audio.clone().info.unwrap().mimetype.unwrap()
					)
				} else { String::new() }
			MessageType::Image(image) => 
				if let MediaSource::Plain(uri) = &image.source {
					format!(
						"<img src=\"{}{}\" type=\"{}\">",
						download_path,
						uri.media_id().unwrap(),
						image.clone().info.unwrap().mimetype.unwrap()
					)
				} else { String::new() }
			MessageType::Text(text) => 
				format!(
					"<p>{}</p>",
					text.body.replace("\n", "<br>"),
				),
			MessageType::Video(video) => 
				if let (
					Some(MediaSource::Plain(thumbnail_source)), 
					MediaSource::Plain(uri)
				) = (matrix_sdk::media::MediaEventContent::thumbnail_source(video), &video.source) {
					format!(
						"<video controls poster=\"{}{}\"><source src=\"{}{}\" type=\"{}\"></video>",
						download_path,
						thumbnail_source.media_id().unwrap(),
						download_path,
						uri.media_id().unwrap(), 
						video.clone().info.unwrap().mimetype.unwrap(),
					)
				} else { String::new() }
			_ => String::new()
		}
	}
}

impl ToHtmlString for Message {
	fn to_html_string(&self) -> String {
		format!(
			"\t\t{}\n\t\t{}\n",
			self.content.msgtype.to_html_string(),
			self.origin_server_ts.to_html_string(),
		)
	}
}
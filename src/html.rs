use chrono::{Local, DateTime};
use matrix_sdk::{ruma::{events::room::{message::MessageType, MediaSource}, MilliSecondsSinceUnixEpoch, OwnedMxcUri}, media::MediaEventContent};

pub trait ToHtml { 
	fn to_html(&self) -> String;
}

impl ToHtml for MilliSecondsSinceUnixEpoch {
	fn to_html(&self) -> String {
		format!(
			"<time>{}</time>", 
			DateTime::<Local>::from(self.to_system_time().unwrap())
				.format("%H:%M, %d %b %Y")
				.to_string()
		)
	}
}

impl ToHtml for MessageType {
	fn to_html(&self) -> String {
		match self {
			MessageType::Audio(audio) => 
				if let MediaSource::Plain(uri) = &audio.source {
					format!(	
						"<audio controls><source src=\"{}\" type=\"{}\"></audio>",
						url(uri), 
						audio.clone().info.unwrap().mimetype.unwrap()
					)
				} else { String::new() }
			MessageType::Image(image) => 
				if let MediaSource::Plain(uri) = &image.source {
					format!(
						"<img src=\"{}\" type=\"{}\">",
						url(uri),
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
				) = (MediaEventContent::thumbnail_source(video), &video.source) {
					format!(
						"<video controls loop poster=\"{}\"><source src=\"{}\" type=\"{}\"></video>",
						url(&thumbnail_source),
						url(uri),
						video.clone().info.unwrap().mimetype.unwrap(),
					)
				} else { String::new() }
			_ => String::new()
		}
	}
}

pub fn url(mxc: &OwnedMxcUri) -> String {
	format!("https://{}/_matrix/media/v3/download/{}/{}", 
		mxc.server_name().unwrap(),
		mxc.server_name().unwrap(), 
		mxc.media_id().unwrap()
	)
}
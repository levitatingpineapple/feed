use chrono::{Local, DateTime};
use matrix_sdk::{ruma::{MilliSecondsSinceUnixEpoch, events::room::{message::MessageType, MediaSource}}, media::MediaEventContent};

pub trait ToHtmlString { 
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
				) = (MediaEventContent::thumbnail_source(video), &video.source) {
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
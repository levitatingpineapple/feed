use actix_web::*;
use handlebars::*;
use html_string::*;
use rss::*;
use chrono::*;
use matrix_sdk::{ruma::{*, events::{*,room::{message::{MessageType, RoomMessageEventContent}}}}, Client};

type Message = OriginalMessageLikeEvent<RoomMessageEventContent>;

pub mod html_string;

struct AppState<'a> {
	client: Client,
	handlebars: Handlebars<'a>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	// Matrix
	let bot = user_id!("@bot:n0g.rip");
	let client = matrix_sdk::Client::builder()
		.server_name(bot.server_name())
		.build()
		.await.unwrap();
	client.login_username(bot, "sorzon-korqi7-sekWug").send()
		.await.unwrap();
	let client_sync = client.clone();
	tokio::spawn(async move { 
		client_sync.sync(matrix_sdk::config::SyncSettings::default()).await
	});

	// Templating
	let mut handlebars = Handlebars::new();
	handlebars.register_template_string("feed", include_str!("feed.html")).unwrap();

	// Webserver
	let data = web::Data::new(
		AppState { client: client, handlebars: handlebars }
	);
	HttpServer::new(move || {
		App::new()
			.app_data(data.clone())
			.route("/feed", web::get().to(feed))
			.route("/rss", web::get().to(rss))
	})
	.bind(("localhost", 5555))?
	.run()
	.await
}

async fn feed(data: web::Data<AppState<'_>>) -> HttpResponse {
	#[derive(::serde::Serialize)]
	struct Page { body: String }
	HttpResponse::Ok().body(
		data.handlebars.render(
			"feed", 
			&Page { 
				body: messages(&data.client).await.iter().map(|m| { 
					format!(
						"\t\t{}\n\t\t{}\n",
						m.content.msgtype.to_html_string(),
						m.origin_server_ts.to_html_string(),
					)
				}).collect::<String>()
			 }
		).unwrap()
	)
}

async fn rss(data: web::Data<AppState<'_>>) -> HttpResponse {
	HttpResponse::Ok()
		.content_type(http::header::ContentType::xml())
		.body(
			ChannelBuilder::default()
			.title("n0g.rip".to_string())
			.link("http://n0g.rip".to_string())
			.description("Feed".to_string())
			.items(
				messages(&data.client).await.iter().map(|m| {
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
						DateTime::<Local>::from(
							m.origin_server_ts
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
		)
}

async fn messages(client: &Client) -> Vec<Message> {
	let mut options = matrix_sdk::room::MessagesOptions::backward();
	options.limit = uint!(100);
	client.get_joined_room(room_id!("!bUtdRxQiBPeYOa3Z:n0g.rip")).unwrap()
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
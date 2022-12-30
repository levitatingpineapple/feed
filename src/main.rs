use actix_web::*;
use handlebars::*;
use matrix_sdk::{Client, ruma::events::room::message::MessageType};
use rss::*;
use chrono::*;

pub mod html;
use html::*;

pub mod matrix;
use matrix::*;

pub mod message;

struct AppState {
	client: Client,
	handlebars: Handlebars<'static>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let data = web::Data::new(
		AppState { 
			client: client().await,
			handlebars: registry()
		}
	);
	HttpServer::new(move || {
		App::new()
			.app_data(data.clone())
			.route("/feed", web::get().to(feed))
			.route("/rss", web::get().to(rss))
			.route("/dm", web::get().to(dm))
			.service(web::resource("/ws").route(web::get().to(ws)))
			
	})
	.bind(("localhost", 5555))?
	.run()
	.await
}

fn registry() -> Handlebars<'static> {
	let mut registry = Handlebars::new();
	registry.register_template_string(
		"feed", 
		include_str!("../static/feed.html")
	).unwrap();
	registry
}

async fn feed(data: web::Data<AppState>) -> HttpResponse {
	#[derive(::serde::Serialize)]
	struct Page { body: String }
	HttpResponse::Ok().body(
		data.handlebars.render(
			"feed", 
			&Page { 
				body: messages(&data.client, FEED).await.iter().map(|m| { 
					format!(
						"\t\t{}\n\t\t{}\n",
						m.content.msgtype.to_html(),
						m.origin_server_ts.to_html(),
					)
				}).collect::<String>()
			 }
		).unwrap()
	)
}

async fn rss(data: web::Data<AppState>) -> HttpResponse {
	HttpResponse::Ok()
		.content_type(http::header::ContentType::xml())
		.body(
			ChannelBuilder::default()
			.title("n0g.rip".to_string())
			.link("http://n0g.rip".to_string())
			.description("Feed".to_string())
			.items(
				messages(&data.client, FEED).await.iter().map(|m| {
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
					let content = Some(m.content.clone().msgtype.to_html());
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

async fn dm() -> HttpResponse {
	HttpResponse::Ok().body(include_str!("../static/dm.html"))
}

async fn ws(data: web::Data<AppState>, http_request: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
	let (response, session, message_stream) = actix_ws::handle(&http_request, stream)?;
	rt::spawn(message::handler(data.client.clone(), session, message_stream));
	Ok(response)
}
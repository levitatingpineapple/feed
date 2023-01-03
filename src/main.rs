use actix_web::{*, http::header::HeaderValue, cookie::Cookie};
use handlebars::*;
use matrix_sdk::{Client, ruma::{events::room::message::MessageType, RoomId}, room::Joined};
use ::rss::*;
use chrono::*;

pub mod html;
use html::*;

pub mod matrix;
use matrix::*;

pub mod chat;

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
			.service(feed)
			.service(rss)
			.service(dm)
			.service(join)
			.service(token)
	})
	.bind(("10.0.0.247", 5555))?
	.run()
	.await
}

#[get("/")]
async fn feed(data: web::Data<AppState>, http_request: HttpRequest) -> HttpResponse {
	#[derive(::serde::Serialize)]
	struct Page {
		body: String,
		button: String
	}
	
	HttpResponse::Ok().body(
		data.handlebars.render(
			"feed", 
			&Page {
				body: messages(&data.client, None).await.iter().map(|m| { 
					format!(
						"\t\t\t{}\n\t\t\t{}\n",
						m.content.msgtype.to_html(),
						m.origin_server_ts.to_html(),
					)
				}).collect::<String>(),
				button: if http_request.get_joined(&data.client).is_some() {
					include_str!("../static/button.html").to_string()
				} else {
					String::new()
				}
			}
		).unwrap()
	)
}

#[get("/rss")]
async fn rss(data: web::Data<AppState>) -> HttpResponse {
	HttpResponse::Ok()
		.content_type(http::header::ContentType::xml())
		.body(
			ChannelBuilder::default()
			.title("n0g.rip".to_string())
			.link("http://n0g.rip".to_string())
			.description("Feed".to_string())
			.items(
				messages(&data.client, None).await.iter().map(|m| {
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

#[get("/dm")]
async fn dm(
	data: web::Data<AppState>, 
	http_request: HttpRequest, 
	payload: web::Payload
) -> Result<HttpResponse, actix_web::Error> {
	match http_request.get_joined(&data.client) {
		Some(joined) => if http_request.headers().get("upgrade") != Some(
			&HeaderValue::from_str("websocket").unwrap()
		) { 
			Ok(HttpResponse::Ok().body(include_str!("../static/chat.html")))
		} else {
			let (response, session, message_stream) = actix_ws::handle(&http_request, payload)?;
			rt::spawn(
				chat::handler(
					data.client.clone(),
					joined,
					session, 
					message_stream
				)
			);
			Ok(response)
		}
		None => Ok(HttpResponse::TemporaryRedirect()
			.append_header(("location", "/"))
			.finish())
	}
}

#[get("/{token}")]
async fn token(data: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
	let mut redirect = HttpResponse::TemporaryRedirect();
	if let Some(joined) = path.into_inner().get_joined(&data.client) {
		redirect.cookie(Cookie::build("token", joined.room_id().localpart()).finish());
	}
	redirect
		.append_header(("location", "/"))
		.finish()
}

#[derive(::serde::Deserialize, Debug)]
struct FormData {
	username: String,
	password: String,
	confirm: String,
}

#[post("/dm")]
async fn join(form: web::Form<FormData>) -> HttpResponse {
	println!("{:?}", form.into_inner());
	
	
	#[derive(::serde::Deserialize, Debug)]
	struct NonceResponse {
		nonce: String
	}
	
	let _client = awc::Client::default()
		.get("localhost:8008/_synapse/admin/v1/register")
		.send()
		.await.unwrap().body();
	
	
	HttpResponse::TemporaryRedirect()
		.append_header(("location", "https://chat.n0g.rip"))
		.finish()
}

// Helpers
fn registry() -> Handlebars<'static> {
	let mut registry = Handlebars::new();
	registry.register_template_string(
		"feed", 
		include_str!("../static/feed.html")
	).unwrap();
	registry
}
trait GetJoined { 
	fn get_joined(&self, client: &Client) -> Option<Joined>;
}

impl GetJoined for String {
	fn get_joined(&self, client: &Client) -> Option<Joined> {
		client.get_joined_room(
			<&RoomId>::try_from(
				format!("!{}:n0g.rip", self,
			).as_str()).ok()?
		)
	}
}

impl GetJoined for HttpRequest {
	fn get_joined(&self, client: &Client) -> Option<Joined> {
		self.cookie("token")
			.and_then(|c| Some(c.value().to_string()))?
			.get_joined(client)
	}
}
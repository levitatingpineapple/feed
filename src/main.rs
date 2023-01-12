use actix_web::{http::{header::HeaderValue, self}, cookie::Cookie, web::{Data, Payload, Path, Form}, HttpServer, App, HttpRequest, HttpResponse, get, rt, post};
use handlebars::*;
use include_dir::{include_dir, Dir};
use matrix_sdk::{Client, ruma::events::room::message::MessageType, room::Joined};
use ::rss::*;
use chrono::*;

pub mod html;
use html::*;

pub mod matrix;
use matrix::*;

pub mod chat;

pub mod signup;
use signup::*;

static RES: Dir = include_dir!("$CARGO_MANIFEST_DIR/res");

struct AppState {
	client: Client,
	handlebars: Handlebars<'static>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let data = Data::new(
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
			.service(lobby)
			.service(join)
			.service(token)
	})
	.bind(("localhost", 5555))?
	.run()
	.await
}

#[get("/")]
async fn feed(data: Data<AppState>, http_request: HttpRequest) -> HttpResponse {
	#[derive(::serde::Serialize)]
	struct Feed {
		body: String,
		chat: String
	}
	
	impl Feed {
		fn new(messages: Vec<Message>, is_chat_visible: bool) -> Feed {
			Feed {
				body: messages.iter().map(|m| { 
					format!(
						"\t\t\t{}\n\t\t\t{}\n",
						m.content.msgtype.to_html(),
						m.origin_server_ts.to_html(),
					)
				}).collect::<String>(),
				chat: if is_chat_visible { "visible" } else { "hidden" }
					.to_string()
			}
		}
	}
	
	HttpResponse::Ok().body(
		data.handlebars.render(
			"feed", 
			&Feed::new(
				messages(&data.client, None).await,
				http_request.get_joined(&data.client).is_some()
			)
		).unwrap()
	)
}

#[get("/rss")]
async fn rss(data: Data<AppState>) -> HttpResponse {
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

#[get("/lobby")]
async fn lobby(
	data: Data<AppState>, 
	http_request: HttpRequest, 
	payload: Payload
) -> Result<HttpResponse, actix_web::Error> {
	match http_request.get_joined(&data.client) {
		
		Some(joined) => if http_request.headers().get("upgrade") != Some(
			&HeaderValue::from_str("websocket").unwrap()
		) { 
			#[derive(::serde::Serialize)]
			struct Chat {
				greeting: String,
				topic: String
			}
			
			impl Chat {
				fn new(joined: Joined) -> Chat {
					Chat {
						greeting: joined.name()
							.filter(|n| !n.is_empty())
							.map_or(String::new(), |n| format!("<h2>Hi, {}!</h2>", n)),
						topic: joined.topic()
							.filter(|t| !t.is_empty())
							.map_or(String::new(), |t| format!("<p>{}</p>", t))
					}
				}
			}
			
			Ok(
				HttpResponse::Ok().body(
					data.handlebars.render("chat", &Chat::new(joined)).unwrap()
				)
			)
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
async fn token(data: Data<AppState>, path: Path<String>) -> HttpResponse {
	// Get resources
	if let Some(resource) = RES.get_file(path.clone()) {
		return HttpResponse::Ok().body(resource.contents())
	}
	
	// Check joined room and cookie-redirect back to feed
	let mut redirect = HttpResponse::TemporaryRedirect();
	if let Some(joined) = path.into_inner().get_joined(&data.client) {
		redirect.cookie(Cookie::build("token", joined.room_id().localpart()).finish());
	}
	redirect
		.append_header(("location", "/"))
		.finish()
}

#[post("/dm")]
async fn join(form: Form<ExternalForm>) -> HttpResponse {
	form.register().await;
	HttpResponse::SeeOther()
		.append_header(("location", "https://chat.n0g.rip"))
		.finish()
}

// Helpers
fn registry() -> Handlebars<'static> {
	let mut registry = Handlebars::new();
	registry.register_template_string(
		"feed", 
		include_str!("../template/feed.html")
	).unwrap();
	registry.register_template_string(
		"chat", 
		include_str!("../template/chat.html")
	).unwrap();
	registry
}
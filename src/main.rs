use actix_web::*;
use handlebars::*;
use serde::Serialize;
mod matrix;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	HttpServer::new(|| App::new()
		.route("/feed", web::get().to(feed))
		.route("/rss", web::get().to(rss))
	).bind(("127.0.0.1", 5555))?.run().await
}

async fn feed(_req: HttpRequest) -> impl Responder {
	let mut handlebars = Handlebars::new();
	handlebars.register_template_string("feed", include_str!("feed.html")).unwrap();
	#[derive(Serialize)]
	struct Page { body: String }
	HttpResponse::Ok().body(
		handlebars.render(
			"feed", 
			&Page { body: matrix::http_body().await }
		).unwrap()
	)
}

async fn rss(_req: HttpRequest) -> HttpResponse {
	HttpResponse::Ok()
		.content_type(http::header::ContentType::xml())
		.body(matrix::rss().await)
}
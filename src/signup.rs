use hex::encode;
use hmac::{Hmac, Mac};
use serde_json::from_slice;
use sha1::{Sha1};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct InternalForm {
	nonce: String,
	username: String,
	password: String,
	admin: bool,
	mac: String,
}


#[derive(Deserialize, Debug)]
pub struct ExternalForm {
	username: String,
	password: String,
	confirm: String,
}

impl ExternalForm {
	
	fn is_valid(&self) -> bool {
		self.password == self.confirm
	}
		
	async fn nonce(&self, client: &awc::Client) -> String {
		#[derive(Deserialize, Debug)]
		struct NonceResponse { nonce: String }
		from_slice::<NonceResponse>(&client
			.get("http://localhost:8008/_synapse/admin/v1/register")
			.send().await.unwrap()
			.body().await.unwrap()
		).unwrap().nonce
	}
	
	async fn mac(&self, nonce: String) -> String {
		let mut hmac = Hmac::<Sha1>::new_from_slice(b"pimDyz-wacwan-6qorno").unwrap();
		hmac.update([
			nonce.as_str(),
			self.username.as_str(),
			self.password.as_str(),
			"notadmin",
		].join("\0").as_bytes());
		encode(hmac.finalize().into_bytes())
	}
	
	async fn internal_form(&self, nonce: String, mac: String) -> InternalForm {
		InternalForm {
			nonce: nonce,
			username: self.username.clone(),
			password: self.password.clone(),
			admin: false,
			mac: mac,
		}
	}
	
	pub async fn register(&self) -> String {
		if !self.is_valid() { panic!("Invalid Form") }
		let client = awc::Client::default();
		let nonce = self.nonce(&client).await;
		let response = client
			.post("http://localhost:8008/_synapse/admin/v1/register")
			.send_json(
				&self.internal_form(
					nonce.clone(),
					self.mac(nonce).await
				).await
			).await;
		match response {
			Ok(response) => format!("Registration response:\n{:#?}", response).to_string(),
			Err(error) => error.to_string()
		}
	}
}
use config::File;
use std::path::Path;
use url::form_urlencoded::{byte_serialize, ByteSerialize};
use webbrowser;
use std::io;

use std::borrow::Cow;

use reqwest::blocking::{Client};
use base64;
use serde::Deserialize;
use reqwest::blocking::{Request, RequestBuilder, Response};

fn url_encode(str: &str) -> String {
    byte_serialize(str.as_bytes()).collect()
}

#[derive(Debug)]
pub struct SpotifyApi{
    client: reqwest::blocking::Client,
    id: String,
    secret: String,
    accessToken: Option<String>,
    refreshToken: Option<String>,
    user_id: Option<String>
}
#[derive(Deserialize, Debug)]
struct SpotifyAuthorizeResponse {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: i32,
    refresh_token: String
}

impl SpotifyApi {
    pub fn new() -> SpotifyApi {
        let mut settings = config::Config::default();

        settings.merge(File::from(Path::new("config.toml"))).unwrap();

        let table = settings.get_table("spotify").unwrap();
        SpotifyApi {
            client : reqwest::blocking::Client::new(),
            id: table["client_id"].to_string(),
            secret: table["client_secret"].to_string(),
            accessToken: None,
            refreshToken: None
        }
    }

    pub fn auth_app(&mut self) {
        let redirect = "https://spotify.com";

        let scopes: String = ["playlist-modify-private", "user-read-currently-playing"].join(" ");

        let url = format!("https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
                          url_encode(self.id.as_str()),
                          url_encode(redirect),
                          url_encode(scopes.as_str())
        );

        webbrowser::open(url.as_ref());

        println!("Input redirect link");

        let mut link: String = "".to_string();
        io::stdin().read_line(&mut link);

        let parsed = url::Url::parse(&link).unwrap();
        let args = parsed.query_pairs();

        let mut code: String = "".to_string();
        for arg in args {
            if *(arg.0) == "code".to_string() {
                code = (*(arg.1)).to_string();
            }
        }

        println!("{}", code);

        let (access_token, refresh_token) = self.get_tokens(&code, redirect, &self.client);

        self.accessToken = Some(access_token);
        self.refreshToken = Some(refresh_token);

        self.get_user_id();
    }

    fn get_tokens(&self, code: &str, redirect: &str, client: &reqwest::blocking::Client) -> (String, String) {
        let url = "https://accounts.spotify.com/api/token";

        let params: [(String, String); 3] = [
            ("grant_type".to_string(), "authorization_code".to_string()),
            ("code".to_string(), code.to_string()),
            ("redirect_uri".to_string(), redirect.to_string())
        ];

        let header = base64::encode(format!("{}:{}", self.id, self.secret));
        let req: reqwest::blocking::Request = client.post(url)
            .header("Authorization", "Basic ".to_string() + &header)
            .form(&params)
            .build()
            .unwrap();

        let resp = client.execute(req);
        let resp : SpotifyAuthorizeResponse = resp.unwrap().json().unwrap();

        (resp.access_token, resp.refresh_token)
    }

    fn add_auth_header(&self, req: RequestBuilder) -> RequestBuilder{
        let value = "Bearer ".to_string() + self.accessToken.as_ref().unwrap();
        req.header("Authorization", value)
    }

    fn get_user_id(&mut self) {

        #[derive(Deserialize, Debug)]
        struct UserResponse {
            id: String
        }

        let url = "https://api.spotify.com/v1/me";
        let mut req: RequestBuilder = self.client.get(url);
        let req = self.add_auth_header(req);

        let resp = self.client.execute(req.build().unwrap()).unwrap();

        let data : UserResponse = resp.json().unwrap();

        self.user_id = Some(data.id);
    }
}
use config::File;
use std::path::Path;
use url::form_urlencoded::{byte_serialize, ByteSerialize};
use webbrowser;
use std::io;

fn url_encode(str: &str) -> String {
    byte_serialize(str.as_bytes()).collect()
}

#[derive(Debug)]
pub struct SpotifyApi{
    client: reqwest::blocking::Client,
    id: String,
    secret: String
}

impl SpotifyApi {
    pub fn new() -> SpotifyApi {
        let mut settings = config::Config::default();

        settings.merge(File::from(Path::new("config.toml"))).unwrap();

        let table = settings.get_table("spotify").unwrap();
        let object = SpotifyApi {
            client : reqwest::blocking::Client::new(),
            id: table["client_id"].to_string(),
            secret: table["client_secret"].to_string()
        };
        object.auth_app();
        object
    }

    fn auth_app(&self) {
        let redirect = "https://spotify.com";

        let scopes: String = ["playlist-modify-private", "user-read-currently-playing"].join(" ");

        let url = format!("https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
                          url_encode(self.id.as_str()),
                          url_encode(redirect),
                          url_encode(scopes.as_str())
        );

        webbrowser::open(url.as_ref());

        println!("Input redirect link");

        let mut link: String;
        io::stdin().read_line(&mut link);


    }
}
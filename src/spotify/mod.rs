use config::File;
use std::path::{Path};
use url::form_urlencoded::{byte_serialize, ByteSerialize};
use webbrowser;
use std::io;

use std::borrow::{Cow, BorrowMut};

use reqwest::blocking::{Client};
use base64;
use serde::Deserialize;
use reqwest::blocking::{Request, RequestBuilder, Response};
use std::fmt;
use serde::export::Formatter;
use std::error::Error;
use std::collections::HashMap;

fn url_encode(str: &str) -> String {
    byte_serialize(str.as_bytes()).collect()
}

#[derive(Debug)]
pub struct SpotifyApi{
    client: reqwest::blocking::Client,
    id: String,
    secret: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
    user_id: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct SpotifyPlaylist {
    pub collaborative: bool,
    pub id : String,
    pub name : String,
    pub public : bool,
    pub snapshot_id : String
}

#[derive(Debug, Deserialize)]
pub struct NameId {
    pub name : String,
    pub id : String
}

impl NameId {
    fn new(name : &str, id : &str) -> NameId {
        NameId {
            name: name.to_string(),
            id: id.to_string()
        }
    }
}

#[derive(Debug)]
pub struct Album {
    pub id: String,
    pub name : String,
    pub artist : Vec<NameId>,
    pub release_date : String,
    pub tracks : Vec<NameId>
}


#[derive(Debug)]
pub struct SpotifyError(String);

impl SpotifyError {
    fn new(msg : &str) -> SpotifyError {
        SpotifyError(msg.to_string())
    }
}

impl fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for SpotifyError {}

impl SpotifyApi {
    pub fn new() -> SpotifyApi {
        let mut settings = config::Config::default();

        settings.merge(File::from(Path::new("config.toml"))).unwrap();

        let table = settings.get_table("spotify").unwrap();
        SpotifyApi {
            client : reqwest::blocking::Client::new(),
            id: table["client_id"].to_string(),
            secret: table["client_secret"].to_string(),
            access_token: None,
            refresh_token: None,
            user_id: None
        }
    }

    pub fn auth_user(&mut self) {

        let redirect = "https://spotify.com";

        let scopes: String = [
            "playlist-modify-private",
            "user-read-currently-playing",
            "playlist-read-private",
            "playlist-modify-public"
        ].join(" ");

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

        let (access_token, refresh_token) = self.get_tokens(&code, redirect, &self.client);

        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);

        self.get_user_id();
    }

    fn get_tokens(&self, code: &str, redirect: &str, client: &reqwest::blocking::Client) -> (String, String) {

        #[derive(Deserialize, Debug)]
        struct SpotifyAuthorizeResponse {
            access_token: String,
            token_type: String,
            scope: String,
            expires_in: i32,
            refresh_token: String
        }

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
        let value = "Bearer ".to_string() + self.access_token.as_ref().unwrap();
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

    pub fn get_playlists_by_name(&self, name: &str) -> Option<SpotifyPlaylist> {
        let playlists : Vec<SpotifyPlaylist> = self.get_all_playlists();

        playlists.into_iter().find(|elem| elem.name == name)
    }

    pub fn get_all_playlists(&self) -> Vec<SpotifyPlaylist> {
        let url = format!("https://api.spotify.com/v1/users/{}/playlists", self.user_id.as_ref().unwrap());
        let mut cur_offset = 0;

        fn get_params(offset: i32) -> [(String, i32);2] {
            [
                ("limit".to_string(), 20),
                ("offset".to_string(), offset)
            ]
        }

        #[derive(Deserialize, Debug)]
        struct UsersPlaylists {
            items: Vec<SpotifyPlaylist>,
            next: Option<i32>
        }

        let mut playlists : Vec<SpotifyPlaylist> = Vec::new();

        loop {
            let mut req = self.client.get(&url);
            let req = self.add_auth_header(req).query(&get_params(cur_offset)).build().unwrap();

            let mut resp  = self.client.execute(req);

            let mut resp_json : UsersPlaylists = resp.unwrap().json().unwrap();

            playlists.append( &mut resp_json.items);

            match resp_json.next {
                None => break,
                Some(_) =>  cur_offset += 20
            };
        }

        return playlists
    }

    pub fn get_album(&self, name : &str, artist: Option<&str>) -> Result<Album, Box<dyn Error>> {
        let url = "https://api.spotify.com/v1/search";

        let req = self.client.get(url);

        let query = match artist {
            None => format!("album:{}", name),
            Some(artistName) => format!("album:{} artist:{}", name, artistName)
        };

        let params1 = [("q", query), ("type", "album".to_string())];

        let params2  = [("limit", 3)];

        let req = self.add_auth_header(req.query(&params1).query(&params2)).build().unwrap();

        let resp = self.client.execute(req);

        #[derive(Deserialize, Debug)]
        struct JsonArtist {
            name : String,
            id : String
        }

        #[derive(Deserialize, Debug)]
        struct JsonAlbum {
            id : String,
            name : String,
            release_date : String,
            artists : Vec<JsonArtist>
        }

        #[derive(Deserialize, Debug)]
        struct JsonItems {
            items : Vec<JsonAlbum>
        }

        #[derive(Deserialize, Debug)]
        struct JsonAlbums {
            albums : JsonItems
        }

        let resp : JsonAlbums = resp?.json()?;

        let mut album = resp.albums.items.first();

        return match album {
            Some(album) => {
                let mut album = Album {
                    id: album.id.clone(),
                    name : album.name.clone(),
                    artist : album.artists.iter().map(|art| NameId::new(&art.name, &art.id)).collect(),
                    release_date: album.release_date.clone(),
                    tracks: vec![]
                };
                self.fill_tracks(&mut album)?;
                Ok(album)
            },
            None => Err(Box::new(SpotifyError("No albumes found".into())))
        }
    }

    fn fill_tracks(&self, album: &mut Album) -> Result<(), Box<dyn Error>> {
        let url = format!("https://api.spotify.com/v1/albums/{}/tracks", album.id);

        let req = self.add_auth_header(self.client.get(&url)).build()?;

        #[derive(Deserialize)]
        struct JsonTracks {
            items : Vec<NameId>
        }

        let resp : JsonTracks = self.client.execute(req)?.json()?;

        album.tracks = resp.items;

        Ok(())
    }

    pub fn add_track_to_playlist(&self, playlist_id: &str, track_ids: &Vec<&str>) -> Result<(), Box<dyn Error>> {
        let url = format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id);

        let tracks_query : Vec<String> = track_ids.iter()
            .map(|id| format!("spotify:track:{}", id)).collect();
        let mut params = HashMap::new();

        params.insert("uris", tracks_query);

        let req = self.client.post(&url)
            .json(&params)
            .header("Content-Type", "application/json");



        let req = self.add_auth_header(req).build().expect("Couldn't build request");

        assert_eq!(self.client.execute(req)?.status().as_u16(), 201);

        Ok(())
    }
}
extern crate reqwest;
extern crate config;

use reqwest::Result;
use std::path::Path;
use std::collections::HashMap;
use config::File;

mod spotify;
mod data;


use spotify::SpotifyApi;

fn main() {

    let mut spotify_api = SpotifyApi::new();

    spotify_api.auth_user();

    let album = spotify_api.get_album("Another World", Some("Gojira"));

    println!("{:?}", album);

    println!("{:?}", spotify_api);
}

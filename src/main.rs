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

    println!("{:?}", spotify_api)
}

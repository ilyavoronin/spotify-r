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

    let spotify_api = SpotifyApi::new();

    spotify_api.auth_app();

    println!("{:?}", spotify_api)
}

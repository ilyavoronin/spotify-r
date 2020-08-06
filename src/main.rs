extern crate reqwest;
extern crate config;

use reqwest::Result;
use std::path::Path;
use std::collections::HashMap;
use config::File;

mod spotify;


use spotify::SpotifyApi;

fn main() {

    let spotify_api = SpotifyApi::new();

    println!("{:?}", spotify_api)
}

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

    let playlist = spotify_api.get_playlists_by_name("test_playlist").unwrap();

    let album = spotify_api.get_album("Another World", Some("Gojira")).unwrap();

    spotify_api.add_track_to_playlist(&playlist.id, &vec![&album.tracks.first().unwrap().id]).expect("AAAAAAAAAAAAAAAAA");
}

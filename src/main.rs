extern crate reqwest;
extern crate config;

use reqwest::Result;
use std::path::Path;
use std::collections::HashMap;
use config::File;

mod spotify;
mod data;
mod update;


use spotify::SpotifyApi;

use update::metal_injection::MetalInjectionUpdater;
use chrono::NaiveDate;
use crate::update::NewAlbumFinder;

fn main() {

    /*
    let mut spotify_api = SpotifyApi::new();

    spotify_api.auth_user();

    let playlist = spotify_api.get_playlists_by_name("test_playlist").unwrap();

    let album = spotify_api.get_album("Another World", Some("Gojira")).unwrap();

    spotify_api.add_track_to_playlist(&playlist.id, &vec![&album.tracks.first().unwrap().id]);
     */

    println!("{:?}", MetalInjectionUpdater::get_new_albums(NaiveDate::from_ymd(2020, 07, 01)));
}

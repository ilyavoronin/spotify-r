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
use crate::update::{NewAlbumFinder, NewAlbumPlaylistUpdater};

fn main() {

    let mut spotify_api = SpotifyApi::new();

    spotify_api.auth_user();

    let updater = NewAlbumPlaylistUpdater::new(
        vec![Box::new(MetalInjectionUpdater{})],
        spotify_api,
        "test_playlist".to_string()
    );

    updater.update_from(9, 8,2020);
}

use chrono::NaiveDate;

use std::error::Error;
use crate::spotify::{SpotifyApi, Album};
use std::borrow::Borrow;

#[derive(Debug)]
pub struct NewAlbum {
    pub name : String,
    pub artist: Option<String>
}

impl NewAlbum {
    pub fn new(name: &str, artist: &str) -> NewAlbum {
        NewAlbum {
            name : name.to_string(),
            artist : Some(artist.to_string())
        }
    }
}

pub mod metal_injection;

pub trait NewAlbumFinder {
    fn get_new_albums(&self, date: NaiveDate) -> Result<Vec<NewAlbum>, Box<dyn Error>>;
}

pub struct NewAlbumPlaylistUpdater {
    updaters : Vec<Box<dyn NewAlbumFinder>>,
    spotify_api : SpotifyApi,
    playlist_name : String
}

impl NewAlbumPlaylistUpdater {
    pub fn new(
        updaters: Vec<Box<dyn NewAlbumFinder>>,
        spotify_api: SpotifyApi,
        playlist_name: String
    ) -> NewAlbumPlaylistUpdater {
        NewAlbumPlaylistUpdater {
            updaters,
            spotify_api,
            playlist_name
        }
    }

    pub fn update_from(&self, day: u32, month: u32, year: i32) {
        let new_albums : Vec<NewAlbum> = self.updaters.iter().flat_map(|updater|
            updater.get_new_albums(NaiveDate::from_ymd(year, month, day)).unwrap()
        ).collect();

        let playlist = self.spotify_api.get_playlists_by_name(&self.playlist_name).unwrap();

        let albums : Vec<Album> = new_albums.iter().filter_map(
            |nalb|
                match self.spotify_api.get_album(&nalb.name, nalb.artist.as_deref()) {
                    Ok(alb) => Some(alb),
                    Err(_) => {
                        match self.spotify_api.get_album(&nalb.name, nalb.artist.as_deref()) {
                            Ok(alb) => {Some(alb)},
                            Err(_) => {
                                println!("Album {} by {} not found", nalb.name, nalb.artist.as_ref().unwrap());
                                None
                            }
                        }
                    }
                }
        ).collect();

        let tracks: Vec<&str> = albums.iter().map(|alb| alb.tracks.first().unwrap().id.borrow()).collect();

        self.spotify_api.add_tracks_to_playlist(&playlist.id, &tracks);
    }
}
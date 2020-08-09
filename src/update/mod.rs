
use chrono::NaiveDate;

use std::error::Error;

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
    fn get_new_albums(date: NaiveDate) -> Result<Vec<NewAlbum>, Box<dyn Error>>;
}
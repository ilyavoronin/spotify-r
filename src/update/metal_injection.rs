use crate::update::{NewAlbumFinder, NewAlbum};
use chrono::NaiveDate;

use reqwest::blocking;

use std::error::Error;
use select::predicate::{Predicate, Attr, Class, Name};
use select::document::Document;
use select::node::Node;

pub struct MetalInjectionUpdater {
}

impl MetalInjectionUpdater {
    const BASE_URL: &'static str =
        "https://metalinjection.net/category/upcoming-releases/heavy-new-releases";

    fn get_article_urls(date: NaiveDate) -> Vec<String> {
        vec![]
    }

    fn get_date_from_string(s: &str) -> NaiveDate {
        let real_date = s.split_at(s.find("|").unwrap()).1;

        let real_date = real_date[1..].trim();

        NaiveDate::parse_from_str(&real_date, "%B %d, %Y").unwrap()
    }

    fn get_pages_url_from_date(date: NaiveDate) -> Result<Vec<String>, Box<dyn Error>> {
        let client = blocking::Client::new();

        let text = client.get(MetalInjectionUpdater::BASE_URL).send()?.text()?;

        let doc = Document::from_read(text.as_bytes()).unwrap();

        let mut res: Vec<String> = vec![];

        for main_node in doc.find(Class("col-md-8")) {
            for elem in main_node.find(Name("article")) {
                let link_node = elem.find(Name("a")).next().unwrap();
                let link = link_node.attr("href").unwrap();

                let date_node = elem.find(Class("meta")).next().unwrap();
                let date_text = date_node.text();

                let article_date = MetalInjectionUpdater::get_date_from_string(&date_text);

                if article_date >= date {
                    res.push(link.to_string());
                }
            }
        }
        Ok(res)
    }

    fn get_new_albums_from_page(page_url: &str) -> Vec<NewAlbum> {
        let client = blocking::Client::new();

        let text = client.get(page_url).send().unwrap().text().unwrap();

        let doc = Document::from_read(text.as_bytes()).unwrap();

        let mut res: Vec<NewAlbum> = vec![];

        let main_elem = doc.find(Class("thearticlecontent")).next().unwrap();

        fn get_album_from_node(elem: Node) -> Option<NewAlbum> {
            let artist_name = elem.first_child().unwrap().text();
            let artist_name = artist_name.trim_end_matches(|e| char::is_whitespace(e) || e == '–');

            let album_name = match elem.find(Name("em")).next() {
                Some(s) => s.first_child().unwrap().text(),
                None => return None
            };

            Some(NewAlbum::new(&album_name, artist_name))
        }

        for elem in main_elem.find(Name("h3")) {
            let alb: Option<NewAlbum> = get_album_from_node(elem);
            match alb {
                Some(na) => res.push(na),
                None => break
            }
        }

        for elem in main_elem.find(Name("ul").child(Name("li"))) {
            let alb: Option<NewAlbum> = get_album_from_node(elem);
            match alb {
                Some(na) => res.push(na),
                None => break
            }
        }

        res
    }
}

impl NewAlbumFinder for MetalInjectionUpdater {
    fn get_new_albums(date: NaiveDate) -> Result<Vec<NewAlbum>, Box<dyn Error>> {

        let urls : Vec<String> = MetalInjectionUpdater::get_pages_url_from_date(date).unwrap();

        Ok(
            urls.iter().flat_map(|url|
                MetalInjectionUpdater::get_new_albums_from_page(url)
            ).collect()
        )
    }
}
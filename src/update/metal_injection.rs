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
        let real_date = s;

        return match NaiveDate::parse_from_str(&real_date, "%B %d, %Y") {
            Ok(res) => res,
            Err(_) => {
                let a: char = s.chars().next().unwrap();
                let days: i64 = a.to_string().parse().unwrap();
                let cur_date: NaiveDate = chrono::Local::now().date().naive_local();
                return cur_date - chrono::Duration::days(days)
            }
        }
    }

    fn get_pages_url_from_date(date: NaiveDate) -> Result<Vec<String>, Box<dyn Error>> {
        let client = blocking::Client::new();

        let text = client.get(MetalInjectionUpdater::BASE_URL).send()?.text()?;

        let doc = Document::from_read(text.as_bytes()).unwrap();

        let mut res: Vec<String> = vec![];

        for main_node in doc.find(Attr("id", "zox-home-cont-wrap")) {
            for elem in main_node.find(Class("zox-art-text")) {
                let title_node = elem.find(Class("zox-art-title")).next().unwrap();
                let link_node = title_node.find(Name("a")).next().unwrap();
                let link = link_node.attr("href").unwrap();

                let date_node = elem.find(Class("zox-byline-date")).next().unwrap();
                let date_text = date_node.text();

                let article_date = MetalInjectionUpdater::get_date_from_string(&date_text);

                if article_date >= date {
                    println!("{}", link);
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

        let main_elem = doc.find(Class("zox-post-body")).next().unwrap();

        fn get_album_from_node(elem: Node) -> Option<NewAlbum> {
            let artist_name = elem.first_child().unwrap().text();
            let artist_name = artist_name.trim_end_matches(|e| char::is_whitespace(e) || e == '–');

            let album_name = match elem.find(Name("em")).next() {
                Some(s) => s.first_child().unwrap().text(),
                None => {
                    println!("Can't parse {}", elem.text());
                    return None;
                }
            };

            Some(NewAlbum::new(&album_name, artist_name))
        }

        for elem in main_elem.find(Name("h3")) {
            let alb: Option<NewAlbum> = get_album_from_node(elem);
            match alb {
                Some(na) => res.push(na),
                None => continue
            }
        }

        for elem in main_elem.find(Name("ul").child(Name("li"))) {
            let alb: Option<NewAlbum> = get_album_from_node(elem);
            match alb {
                Some(na) => res.push(na),
                None => continue
            }
        }

        res
    }
}

impl NewAlbumFinder for MetalInjectionUpdater {
    fn get_new_albums(&self, date: NaiveDate) -> Result<Vec<NewAlbum>, Box<dyn Error>> {

        let urls : Vec<String> = MetalInjectionUpdater::get_pages_url_from_date(date).unwrap();

        Ok(
            urls.iter().flat_map(|url|
                MetalInjectionUpdater::get_new_albums_from_page(url)
            ).collect()
        )
    }
}
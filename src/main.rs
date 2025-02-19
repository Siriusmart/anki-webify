use std::{
    collections::HashMap,
    env, fs,
    io::{Cursor, Write},
    path::PathBuf,
    process,
};

use rusqlite::Connection;
use serde::Deserialize;

fn main() {
    if env::args().count() < 3 {
        println!("No file path specified: `anki-webify [input] [name] (output) (media prepend)`");
        process::exit(-1);
    }

    let path = env::args().nth(1).unwrap();
    let path = PathBuf::from(path);

    if !path.exists() {
        println!("Anki archive not found at file");
        process::exit(-2);
    }

    let id = env::args().nth(2).unwrap();
    let output = env::args().nth(3).unwrap_or_default();
    let output = PathBuf::from(output).join(&id);
    let temp = output.join("temp");

    let media_prepend = env::args().nth(4).unwrap_or("./".to_string());

    if temp.exists() {
        fs::remove_dir_all(&temp).unwrap();
    }

    if !temp.exists() {
        fs::create_dir_all(&output).unwrap();
    }

    let bytes = fs::read(path).unwrap();

    zip_extract::extract(Cursor::new(bytes), &output.join("temp"), true).expect("unzip failed");

    let db = temp.join("collection.anki21");
    let media = temp.join("media");

    if !db.exists() {
        println!(
            "collection.anki21 does not exist, decks should be exported in compatibility mode"
        );
        process::exit(-3);
    }

    let media = fs::read_to_string(media).unwrap();
    let media: HashMap<String, String> = serde_json::from_str(&media).unwrap();
    let media_replace: HashMap<String, String> = media
        .iter()
        .map(|(k, v)| {
            (
                format!(r#"<img src="{v}">"#),
                format!(r#"<img src="{media_prepend}{id}/media/{k}">"#),
            )
        })
        .collect();

    let db = Connection::open(db).unwrap();

    #[derive(Deserialize)]
    struct Deck {
        pub name: String,
    }

    let decks: HashMap<String, String> = {
        let mut stmt = db
            .prepare(
                "SELECT decks
            FROM col",
            )
            .unwrap();
        stmt.query_row([], |row| {
            let map: HashMap<String, Deck> =
                serde_json::from_str(&row.get::<usize, String>(0).unwrap()).unwrap();
            Ok(map.into_iter().map(|(k, v)| (k, v.name)).collect())
        })
        .unwrap()
    };

    #[derive(Deserialize, Debug)]
    struct Card {
        pub front: String,
        pub back: String,
        pub deck: u64,
        pub id: u64,
    }

    let cards: Vec<Card> = {
        let mut stmt = db
            .prepare(
                "SELECT did, flds, notes.id
                FROM notes, cards
                WHERE notes.id == cards.id
                ORDER BY due ASC",
            )
            .unwrap();
        stmt.query_map([], |row| {
            let mut flds: String = row.get(1).unwrap();
            media_replace
                .iter()
                .for_each(|(k, v)| flds = flds.replace(k, v));
            let (front, back) = flds.split_once("").unwrap();
            Ok(Card {
                front: front.to_string(),
                back: back.to_string(),
                deck: row.get(0).unwrap(),
                id: row.get(2).unwrap(),
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };

    fs::create_dir(output.join("front")).unwrap();
    fs::create_dir(output.join("back")).unwrap();

    let mut index: HashMap<String, Vec<u64>> = HashMap::new();

    for card in cards {
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output.join("front").join(card.id.to_string()))
            .unwrap()
            .write_all(card.front.as_bytes())
            .unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output.join("back").join(card.id.to_string()))
            .unwrap()
            .write_all(card.back.as_bytes())
            .unwrap();

        let deck_name = decks.get(&card.deck.to_string()).unwrap();
        index
            .entry(deck_name.to_string())
            .or_default()
            .push(card.id);
    }

    fs::create_dir_all(output.join("media")).unwrap();

    for k in media.keys() {
        fs::rename(temp.join(k), output.join("media").join(k)).unwrap();
    }

    fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output.join("index.json"))
        .unwrap()
        .write_all(&serde_json::to_vec(&index).unwrap())
        .unwrap();

    fs::remove_dir_all(temp).unwrap();
}

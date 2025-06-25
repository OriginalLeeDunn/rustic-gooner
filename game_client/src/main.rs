use bevy::prelude::*;
use reqwest::blocking::get;

fn fetch_from_server() {
    let response = get("http://localhost:8000").unwrap().text().unwrap();
    println!("Server says: {}", response);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, fetch_from_server)
        .run();
}

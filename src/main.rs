#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate web_view;

use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;
use web_view::*;
use crate::Cmd::Save;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Save { text: String, file_name: String },
}
const INDEX: &str = include_str!("index.html");

fn main() {
    web_view::builder()
        .title("BEE TXT")
        .content(Content::Html(INDEX))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            handler(webview, arg)
        })
        .run()
        .unwrap()
}

fn handler(_webview: &mut web_view::WebView<'_, ()>, arg: &str )
    -> WVResult {
    match serde_json::from_str(arg).unwrap() {
        Save { text, file_name } => {
            let file = PathBuf::from(file_name);
            let mut save_file = File::create(&file).unwrap();
            save_file.write_all(text.as_bytes()).unwrap();
        },
    }
    Ok(())
}
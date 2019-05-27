#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate web_view;

use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;
use web_view::*;
use crate::Cmd::*;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Save { file: String, content: String },
    Quit {  },
}

const INDEX: &str = include_str!("index.html");

fn main() {
    let mut webview = web_view::builder()
        .title("")
        .content(Content::Html(INDEX))
        .size(500, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg|  {
            handler(webview, arg)
        })
        .build()
        .unwrap();

    webview.set_color((37,21,85));
    webview.run().unwrap();
}

fn handler(_webview: &mut web_view::WebView<'_, ()>, arg: &str )
    -> WVResult {
    match serde_json::from_str(arg).unwrap() {
        Save { file, content } => {
            let mut save_file = File::create(&file).unwrap();
            save_file.write_all(content.as_bytes()).unwrap();
        },
        Quit {} => std::process::exit(0x0100),
    }
    Ok(())
}
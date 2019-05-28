#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate web_view;
extern crate config;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use web_view::*;
use crate::Cmd::*;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Save { file: String, content: String },
    Css { },
    Quit { },
    Open { },
}

//const INDEX: &str = include_str!("index.html");

fn main() {
    let (bg_color, css_colors) = get_colors();
    let css: &str = "html, body {
overflow: hidden;
margin: 0;
padding: 0;
background-color: var(--bg) ;
width: 100%;
height: 100%;
}
main {
display: flex;
height: 97%;
}
textarea, #file {
padding-left: 10px;
padding-top: 3px;
font-family: 'Menlo';
background-color: var(--bg);
border: 0;
}
#file {
font-size: 14px;
font-weight: bolder;
padding-right: 5px;
display: inline-block;
color: var(--title);
}
header {
display: flex;
flex-direction: row;
}
textarea {
color: var(--text);
width: 100%;
height: 100%;
padding-right: 10px;
-webkit-box-sizing: border-box;
box-sizing: border-box;
}
:focus {
outline: 0;
}
::selection {
background-color: var(--hl);
}
#unsaved {
font-size: 11px;
}";
    let html: &str = &format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
      <style>
        {}
      </style>
    </head>
    <body>
      <script>{}</script>
    <header>
      <span type="text" id="file" contenteditable="true">note title.txt</span>
      <div id="unsaved"></div>
    </header>
    <main><textarea rows="8" cols="80" id="content"
    oninput="unsave()"></textarea></main>
    </body>
    </html>"#, format!("{}{}", css, &css_colors), JS);

    let mut webview = web_view::builder()
        .title("")
        .content(Content::Html(html))
        .size(500, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg|  {
            handler(webview, arg)
        })
        .build()
        .unwrap();

    webview.set_color((bg_color[0], bg_color[1], bg_color[2]));
    webview.run().unwrap();
}

fn handler(webview: &mut web_view::WebView<'_, ()>, arg: &str )
    -> WVResult {
    match serde_json::from_str(arg).unwrap() {
        Open { } => {
            let path_buf = webview.dialog()
                                  .open_file("note", "note_title.txt")
                                  .unwrap();
            match path_buf {
                Some(pb) => {
                    let file_name = pb.to_str().unwrap();
                    let file = File::open(&pb).unwrap();
                    let mut buf_reader = BufReader::new(file);
                    let mut contents = String::new();
                    match buf_reader.read_to_string(&mut contents) {
                        Ok(_) => {
                            webview.eval(&format!("open_file({}, {});",
                                        web_view::escape(file_name),
                                        web_view::escape(&contents)))
                                   .unwrap();
                            },
                        Err(_) => webview.eval(&format!("open_error();")).unwrap()
                    };
                },
                None => ()
            }
        }
        Save { file, content } => {
            let mut save_file = File::create(&file).unwrap();
            save_file.write_all(content.as_bytes()).unwrap();
        },
        Quit {} => std::process::exit(0x0100),
        Css {} => webview.eval(&format!("inject_css({})",&get_colors().1)).unwrap(),
    }
    Ok(())
}

fn get_colors() -> (Vec<u8>, String)  {
    let mut colors = config::Config::default();
    colors.merge(config::File::with_name("colors")).unwrap();
    let text_color = colors.get_str("text").unwrap();
    let title_color = colors.get_str("title").unwrap();
    let hl_color = colors.get_str("hl").unwrap();
    let bg_color = colors.get_str("bg").unwrap();

// parse background color into webview-readable format for title bar
    let bg_color_wv: String = bg_color.chars()
                              .filter(|a| ((a).is_numeric() || a==&','))
                              .collect();
    let bg_vec: Vec<u8> = bg_color_wv.split(',').map(|a| a.parse::<u8>().unwrap()).collect();

// set all colors as css rules
    let css_colors = format!(":root {{--bg: {};--text: {};--title: {};--hl: {};}}",
                                bg_color, text_color, title_color, hl_color);
    (bg_vec, css_colors)
}

const JS: &str = "
document.onkeydown = KeyPress;
var fileIn;
var contentIn;
var savedIndicator;
var sIupdated = true;
var beeemoji = '\\u{1F41D}';

window.onload = () => {
fileIn = document.querySelector('#file');
contentIn = document.querySelector('#content');
savedIndicator = document.querySelector('#unsaved');
savedIndicator.innerText = beeemoji;
}

function KeyPress(e) {
var evtobj = window.event? event : e;

// send various commands to rust
if (evtobj.keyCode == 83 && evtobj.metaKey){
save();
} else if (evtobj.keyCode == 9) {
evtobj.preventDefault();
insertTab(document.activeElement);
} else if ((evtobj.keyCode == 81 || evtobj.keyCode == 87) && evtobj.metaKey) {
external.invoke(JSON.stringify({ cmd: 'quit' }));
} else if (evtobj.keyCode == 79 && evtobj.metaKey){
external.invoke(JSON.stringify({ cmd: 'open' }));
}
}

function save() {
let file = fileIn.innerText.replace(/ /g,'_');
let content = contentIn.value;
let msgJSON = { cmd: 'save', file: file, content: content };
external.invoke(JSON.stringify(msgJSON));
savedIndicator.style.display = 'none';
sIupdated = false;
}

function open_file(file, contents){
fileIn.innerText = file;
contentIn.value = contents;
savedIndicator.style.display = 'none';
sIupdated = false;
}
function open_error(){
alert('file unreadable - sorry :/ ' + beeemoji);
}
function insertTab(field){
let start = field.selectionStart;
let newPos = start + '    '.length;
field.value = field.value.substring(0, start) + '    '
            + field.value.substring(start, field.value.length);
field.selectionStart = newPos;
field.selectionEnd = newPos;
field.focus;
return false;
}

// update save indicator
function unsave() {
if (!sIupdated){
savedIndicator.style.display = 'block';
sIupdated = true;
}}
";
mod lib;
use lib::*;
use askama::Template;
use pulldown_cmark::{Parser, Options, html};



// main関数はwasmでは不要なのでcfgで分岐
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let md = include_str!("../README.md");
    let rendered = lib::process_markdown(md);
    std::fs::write("output.html", rendered).unwrap();
}

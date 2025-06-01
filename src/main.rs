mod lib;
use lib::*;
use askama::Template;



// main関数はwasmでは不要なのでcfgで分岐
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let md = include_str!("./sample.md");
    let (front_matter, title, content) = split_front_matter(md);
    let clean_title = title.trim_start_matches('#').trim();
    let html_output = markdown::to_html(content);
    let wrapped = wrap_head_sections_nested(&html_output);
    // タグをTagLink構造体に変換
    let tags = front_matter.tags.as_ref().map(|tags| {
        tags.iter().map(|tag| TagLink {
            name: tag,
            url: format!("/tags/{}.html", tag),
        }).collect::<Vec<_>>()
    }).unwrap_or_default();
    let template = HtmlTemplate {
        title: clean_title,
        description: front_matter.description.as_deref().unwrap_or("no description"),
        url: "https://example.com/sample",
        image: "https://example.com/ogp.png",
        body: &wrapped,
        tags,
    };
    let rendered = template.render().unwrap();
    std::fs::write("output.html", rendered).unwrap();
}

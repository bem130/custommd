mod lib;
use lib::*;
use askama::Template;
use pulldown_cmark::{Parser, Options, html};



// main関数はwasmでは不要なのでcfgで分岐
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let md = include_str!("../README.md");
    let (front_matter, title, content) = split_front_matter(md);
    let clean_title = title.trim_start_matches('#').trim();
    let mut html_output = String::new();
    let parser = Parser::new_ext(content, Options::all());
    html::push_html(&mut html_output, parser);
    let wrapped = wrap_head_sections_nested(&html_output);

    // --- 階層構造TOC生成 ---
    use regex::Regex;
    let re = Regex::new(r#"<h([1-6])[^>]*>(.*?)</h[1-6]>"#).unwrap();
    #[derive(Debug)]
    struct TocItem {
        level: usize,
        text: String,
        id: String,
        children: Vec<TocItem>,
    }
    let mut stack: Vec<TocItem> = vec![TocItem { level: 0, text: String::new(), id: String::new(), children: vec![] }];
    for cap in re.captures_iter(&html_output) {
        let level: usize = cap[1].parse().unwrap_or(1);
        let text = cap[2].replace("<code>", "").replace("</code>", "");
        let id = text.replace(" ", "-").to_lowercase();
        let item = TocItem { level, text: text.clone(), id, children: vec![] };
        while stack.last().unwrap().level >= level {
            let child = stack.pop().unwrap();
            stack.last_mut().unwrap().children.push(child);
        }
        stack.push(item);
    }
    while stack.len() > 1 {
        let child = stack.pop().unwrap();
        stack.last_mut().unwrap().children.push(child);
    }
    fn render_toc(items: &Vec<TocItem>) -> String {
        if items.is_empty() { return String::new(); }
        let mut html = String::from("<ul class=\"toc-list\">");
        for item in items.iter() {
            if !item.text.is_empty() {
                html.push_str(&format!("<li class=\"toc-level{}\"><a href=\"#{}\">{}</a>", item.level, item.id, item.text));
                html.push_str(&render_toc(&item.children));
                html.push_str("</li>");
            }
        }
        html.push_str("</ul>");
        html
    }
    let toc_html = render_toc(&stack[0].children);

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
        toc: toc_html,
    };
    let rendered = template.render().unwrap();
    std::fs::write("output.html", rendered).unwrap();
}

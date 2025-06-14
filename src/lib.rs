use askama::Template;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use std::fmt::{self, Write};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
pub struct FrontMatter {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub fn split_front_matter(md: &str) -> (FrontMatter, String, &str) {
    let mut lines = md.lines();
    // 1行目: タイトル
    let title = lines.next().unwrap_or("").trim().to_string();
    // 2行目以降: description（空行またはtags:まで）
    let mut description_lines = Vec::new();
    let mut tags = Vec::new();
    let mut in_tags = false;
    let mut body_lines = Vec::new();
    let mut found_tags = false;
    let mut body_flag = false;
    for line in lines {
        let trimmed = line.trim();
        if body_flag {
            body_lines.push(line);
            continue;
        }
        if !in_tags && trimmed == "tags:" {
            in_tags = true;
            found_tags = true;
            continue;
        }
        if in_tags {
            if trimmed.starts_with("-") {
                tags.push(trimmed.trim_start_matches("-").trim().to_string());
                continue;
            } else if !trimmed.is_empty() {
                in_tags = false;
            } else {
                body_flag = true;
                continue;
            }
        }
        if !found_tags && !in_tags {
            if trimmed.is_empty() {
                found_tags = true;
                continue;
            }
            description_lines.push(line);
            continue;
        }
    }
    let description = if description_lines.is_empty() {
        None
    } else {
        Some(description_lines.join("\n").trim().to_string())
    };
    let fm = FrontMatter {
        description,
        tags: if tags.is_empty() { None } else { Some(tags) },
    };
    // 1行目タイトルも返す場合は _title を返すように変更可能
    (fm, title, Box::leak(body_lines.join("\n").into_boxed_str()) as &str)
}

#[derive(Debug)]
pub enum Section {
    Root(Vec<Section>),
    Section {
        level: usize,
        head: String,
        children: Vec<Section>,
    },
    Content(String),
}

impl Section {
    pub fn fmt_with_indent(&self, f: &mut String, indent: usize) {
        let indent_str = |n| "    ".repeat(n);
        match self {
            Section::Root(children) => {
                for child in children {
                    child.fmt_with_indent(f, indent);
                }
            }
            Section::Section { head, children, .. } => {
                writeln!(f, "{}<div class=\"section\">", indent_str(indent)).unwrap();
                writeln!(f, "{}{}", indent_str(indent + 1), head.trim()).unwrap();
                for child in children {
                    child.fmt_with_indent(f, indent + 1);
                }
                writeln!(f, "{}</div>", indent_str(indent)).unwrap();
            }
            Section::Content(s) => {
                let trimmed = s.trim();
                if trimmed.starts_with("<pre><code") && trimmed.ends_with("</code></pre>") {
                    // コードブロックはそのまま出力（インデントやトリムしない）
                    writeln!(f, "{}{}", indent_str(indent), trimmed).unwrap();
                } else {
                    for line in trimmed.lines() {
                        writeln!(f, "{}{}", indent_str(indent), line.trim()).unwrap();
                    }
                }
            }
        }
    }
}

pub fn wrap_head_sections_nested(html: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r#"<h([1-6])([^>]*)>.*?</h[1-6]>"#).unwrap();
    let mut heads = vec![];
    for cap in re.captures_iter(html) {
        let m = cap.get(0).unwrap();
        let level: usize = cap[1].parse().unwrap();
        heads.push((m.start(), m.end(), level));
    }
    let mut stack: Vec<Section> = vec![Section::Root(vec![])];
    let mut last_end = 0;
    let mut i = 0;
    while i < heads.len() {
        let (start, end, level) = heads[i];
        // 直前のhead~今回のheadまでの間の内容
        if last_end < start {
            let content = &html[last_end..start];
            let mut remain = content;
            let delim = "<p>;;;</p>";
            while let Some(idx) = remain.find(delim) {
                let before = &remain[..idx];
                if !before.trim().is_empty() {
                    if let Some(Section::Section { children, .. }) = stack.last_mut() {
                        children.push(Section::Content(before.to_string()));
                    } else if let Some(Section::Root(children)) = stack.last_mut() {
                        children.push(Section::Content(before.to_string()));
                    }
                }
                // sectionをpopして親にpush
                if stack.len() > 1 {
                    let section = stack.pop().unwrap();
                    if let Some(Section::Section { children, .. }) = stack.last_mut() {
                        children.push(section);
                    } else if let Some(Section::Root(children)) = stack.last_mut() {
                        children.push(section);
                    }
                }
                remain = &remain[idx + delim.len()..];
            }
            if !remain.trim().is_empty() {
                if let Some(Section::Section { children, .. }) = stack.last_mut() {
                    children.push(Section::Content(remain.to_string()));
                } else if let Some(Section::Root(children)) = stack.last_mut() {
                    children.push(Section::Content(remain.to_string()));
                }
            }
        }
        let head_html = &html[start..end];
        // スタックの深さ調整
        while stack.len() > 1 {
            if let Some(Section::Section { level: top_level, .. }) = stack.last() {
                if *top_level >= level {
                    let section = stack.pop().unwrap();
                    if let Some(Section::Section { children, .. }) = stack.last_mut() {
                        children.push(section);
                    } else if let Some(Section::Root(children)) = stack.last_mut() {
                        children.push(section);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // 新しいセクション開始
        stack.push(Section::Section {
            level,
            head: head_html.to_string(),
            children: vec![],
        });
        last_end = end;
        i += 1;
    }
    // 残りの内容
    if last_end < html.len() {
        let content = &html[last_end..];
        let mut remain = content;
        let delim = "<p>;;;</p>";
        while let Some(idx) = remain.find(delim) {
            let before = &remain[..idx];
            if !before.trim().is_empty() {
                if let Some(Section::Section { children, .. }) = stack.last_mut() {
                    children.push(Section::Content(before.to_string()));
                } else if let Some(Section::Root(children)) = stack.last_mut() {
                    children.push(Section::Content(before.to_string()));
                }
            }
            if stack.len() > 1 {
                let section = stack.pop().unwrap();
                if let Some(Section::Section { children, .. }) = stack.last_mut() {
                    children.push(section);
                } else if let Some(Section::Root(children)) = stack.last_mut() {
                    children.push(section);
                }
            }
            remain = &remain[idx + delim.len()..];
        }
        if !remain.trim().is_empty() {
            if let Some(Section::Section { children, .. }) = stack.last_mut() {
                children.push(Section::Content(remain.to_string()));
            } else if let Some(Section::Root(children)) = stack.last_mut() {
                children.push(Section::Content(remain.to_string()));
            }
        }
    }
    // スタックを全部閉じる
    while stack.len() > 1 {
        let section = stack.pop().unwrap();
        if let Some(Section::Section { children, .. }) = stack.last_mut() {
            children.push(section);
        } else if let Some(Section::Root(children)) = stack.last_mut() {
            children.push(section);
        }
    }
    // 整形出力
    let mut buf = String::new();
    stack[0].fmt_with_indent(&mut buf, 2);
    buf
}

pub struct TagLink<'a> {
    pub name: &'a str,
    pub url: String,
}

#[derive(Template)]
#[template(path = "template.html")]
pub struct HtmlTemplate<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub url: &'a str,
    pub image: &'a str,
    pub body: &'a str,
    pub tags: Vec<TagLink<'a>>,
    pub toc: String,
}

#[wasm_bindgen]
pub fn process_markdown(md: &str) -> String {
    let (front_matter, title, content) = split_front_matter(md);
    let clean_title = title.trim_start_matches('#').trim();
    // pulldown-cmark でMarkdown→HTML変換
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
    template.render().unwrap_or_else(|_| "<p>テンプレートエラー</p>".to_string())
}
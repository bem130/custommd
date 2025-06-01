use serde::Deserialize;
use std::fmt::{self, Write};

#[derive(Debug, Deserialize)]
struct FrontMatter {
    tags: Option<Vec<String>>,
}

fn split_front_matter(md: &str) -> (Option<FrontMatter>, &str) {
    let mut lines = md.lines();
    let first = lines.next();
    if first == Some("---") {
        let mut yaml = String::new();
        for line in &mut lines {
            if line.trim() == "---" {
                break;
            }
            yaml.push_str(line);
            yaml.push('\n');
        }
        let rest = lines.collect::<Vec<_>>().join("\n");
        let fm: Result<FrontMatter, _> = serde_yaml::from_str(&yaml);
        (fm.ok(), Box::leak(rest.into_boxed_str()) as &str)
    } else {
        (None, md)
    }
}

#[derive(Debug)]
enum Section {
    Root(Vec<Section>),
    Section {
        level: usize,
        head: String,
        children: Vec<Section>,
    },
    Content(String),
}

impl Section {
    fn fmt_with_indent(&self, f: &mut String, indent: usize) {
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
                for line in s.trim().lines() {
                    if !line.trim().is_empty() {
                        writeln!(f, "{}{}", indent_str(indent), line.trim()).unwrap();
                    }
                }
            }
        }
    }
}

fn wrap_head_sections_nested(html: &str) -> String {
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
                if stack.len() > 2 {
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
            if stack.len() > 2 {
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
    stack[0].fmt_with_indent(&mut buf, 0);
    buf
}

fn main() {
    let md = include_str!("./sample.md");
    let (front_matter, content) = split_front_matter(md);
    if let Some(fm) = &front_matter {
        println!("Front matter: {:?}", fm);
    }
    let html_output = markdown::to_html(content);
    let mut wrapped = wrap_head_sections_nested(&html_output);
    wrapped.push_str("<link rel='stylesheet' href='./src/style.css' />");
    std::fs::write("output.html", wrapped).unwrap();
}

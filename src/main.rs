use serde::Deserialize;

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

fn wrap_head_sections_nested(html: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r#"<h([1-6])([^>]*)>.*?</h[1-6]>"#).unwrap();
    let mut result = String::new();
    let mut stack: Vec<(usize, String)> = Vec::new(); // (level, content)
    let mut last_end = 0;
    let mut heads = vec![];
    for cap in re.captures_iter(html) {
        let m = cap.get(0).unwrap();
        let level: usize = cap[1].parse().unwrap();
        heads.push((m.start(), m.end(), level));
    }
    let mut i = 0;
    while i < heads.len() {
        let (start, end, level) = heads[i];
        // 直前のhead~今回のheadまでの間の内容
        if last_end < start {
            let content = &html[last_end..start];
            // ここで;;;を検出してsectionをpopする
            let mut remain = content;
            let delim = "<p>;;;</p>";
            while let Some(idx) = remain.find(delim) {
                let before = &remain[..idx];
                if !before.trim().is_empty() {
                    if let Some((_lvl, ref mut buf)) = stack.last_mut() {
                        buf.push_str(before);
                    } else {
                        result.push_str(before);
                    }
                }
                // sectionをpopしてdivでラップ
                if let Some((_popped_level, popped_content)) = stack.pop() {
                    let div = format!("<div class=\"section\">{}</div>", popped_content);
                    if let Some((_parent_level, ref mut parent_content)) = stack.last_mut() {
                        parent_content.push_str(&div);
                    } else {
                        result.push_str(&div);
                    }
                }
                remain = &remain[idx + delim.len()..];
            }
            // 残り
            if !remain.trim().is_empty() {
                if let Some((_lvl, ref mut buf)) = stack.last_mut() {
                    buf.push_str(remain);
                } else {
                    result.push_str(remain);
                }
            }
        }
        let head_html = &html[start..end];
        // スタックの深さ調整
        while let Some(&(top_level, _)) = stack.last() {
            if top_level >= level {
                let (_popped_level, popped_content) = stack.pop().unwrap();
                let div = format!("<div class=\"section\">{}</div>", popped_content);
                if let Some((_parent_level, ref mut parent_content)) = stack.last_mut() {
                    parent_content.push_str(&div);
                } else {
                    result.push_str(&div);
                }
            } else {
                break;
            }
        }
        // 新しいセクション開始
        stack.push((level, head_html.to_string()));
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
                if let Some((_lvl, ref mut buf)) = stack.last_mut() {
                    buf.push_str(before);
                } else {
                    result.push_str(before);
                }
            }
            // sectionをpopしてdivでラップ
            if let Some((_popped_level, popped_content)) = stack.pop() {
                let div = format!("<div class=\"section\">{}</div>", popped_content);
                if let Some((_parent_level, ref mut parent_content)) = stack.last_mut() {
                    parent_content.push_str(&div);
                } else {
                    result.push_str(&div);
                }
            }
            remain = &remain[idx + delim.len()..];
        }
        // 残り
        if !remain.trim().is_empty() {
            if let Some((_lvl, ref mut buf)) = stack.last_mut() {
                buf.push_str(remain);
            } else {
                result.push_str(remain);
            }
        }
    }
    // スタックを全部閉じる
    while let Some((_lvl, content)) = stack.pop() {
        let div = format!("<div class=\"section\">{}</div>", content);
        if let Some((_parent_level, ref mut parent_content)) = stack.last_mut() {
            parent_content.push_str(&div);
        } else {
            result.push_str(&div);
        }
    }
    result
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

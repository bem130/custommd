use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FrontMatter {
    title: Option<String>,
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

fn main() {
    let md = include_str!("./sample.md");
    let (front_matter, content) = split_front_matter(md);
    if let Some(fm) = &front_matter {
        println!("Front matter: {:?}", fm);
    }
    let html_output = markdown::to_html(content);
    std::fs::write("output.html", html_output).unwrap();
}

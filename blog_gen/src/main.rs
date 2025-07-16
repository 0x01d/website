use serde::{Deserialize, Serialize};
use std::fs::{self};
use glob::glob;
use std::collections::HashMap;


#[derive(Debug, Serialize, Deserialize)]
struct FrontMatter {
    title: String,
    slug: String,
    tags: Vec<String>,
    date: String,
}

#[derive(Debug, Serialize)]
struct BlogEntry {
    title: String,
    slug: String,
    tags: Vec<String>,
    date: String,
}

fn main() {
    let mut entries = Vec::new();
    let mut tags: HashMap<String, i32> = HashMap::new();
    for entry in glob("public/blogs/*.md").unwrap() {
        let path = entry.unwrap();
        println!("Parsing: {}", path.display());
        let content = fs::read_to_string(&path).unwrap();

        if let Some((fm, body)) = extract_frontmatter(&content) {
            let blog_entry = BlogEntry {
                title: fm.title,
                slug: fm.slug.clone(),
                tags: fm.tags.clone(),
                date: fm.date,
            };
            for tag in fm.tags {
                *tags.entry(tag).or_insert(0) += 1;
            }
            entries.push(blog_entry);
        }
    }

    let json_entries = serde_json::to_string_pretty(&entries).unwrap();
    let json_tags = serde_json::to_string_pretty(&tags).unwrap();
    fs::write("public/blogs/index.json", json_entries.as_bytes()).unwrap();
    fs::write("public/blogs/tags.json", json_tags.as_bytes()).unwrap();
}

fn extract_frontmatter(content: &str) -> Option<(FrontMatter, String)> {
    let mut lines = content.lines();

    if lines.next()? != "---" {
        return None;
    }

    let mut frontmatter = String::new();
    for line in &mut lines {
        if line == "---" {
            break;
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }

    let rest = lines.collect::<Vec<&str>>().join("\n");
    let parsed: FrontMatter = serde_yaml::from_str(&frontmatter).ok()?;
    Some((parsed, rest))
}

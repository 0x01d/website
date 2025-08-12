use serde::{Deserialize, Serialize};
use std::fs::{self};
use glob::glob;
use std::collections::HashMap;
use chrono::NaiveDate;


#[derive(Debug, Serialize, Deserialize)]
struct FrontMatter {
    title: String,
    tags: Vec<String>,
    date: NaiveDate,
}

#[derive(Debug, Serialize)]
struct BlogEntry {
    title: String,
    slug: String,
    tags: Vec<String>,
    date: NaiveDate,
}

#[derive(Debug, Serialize)]
struct Tag {
    name: String,
    count: i32,
}

fn main() {
    let mut entries = Vec::new();
    let mut tags: HashMap<String, i32> = HashMap::new();
    tags.insert("All".to_string(), 0);
    for entry in glob("../blogs/*.md").unwrap() {
        let path = entry.unwrap();
        let slug = path.file_name().unwrap().to_string_lossy().into_owned();
        println!("Parsing: {}", path.display());
        let content = fs::read_to_string(&path).unwrap();

        if let Some((fm, body)) = extract_frontmatter(&content) {
            let blog_entry = BlogEntry {
                title: fm.title,
                slug: slug.clone(),
                tags: fm.tags.clone(),
                date: fm.date,
            };
            for tag in fm.tags {
                *tags.entry(tag).or_insert(0) += 1;
            }
            entries.push(blog_entry);
            *tags.entry("All".to_string()).or_insert(0) += 1;
            fs::write(format!("../public/blogs/{}", slug), body.as_bytes()).unwrap();
        }
    }

    let mut tags_vec = Vec::new();

    if let Some(count) = tags.remove("All") {
        tags_vec.push(Tag {
            name: "All".to_string(),
            count,
        });
    }

    for (name, count) in tags {
        tags_vec.push(Tag { name, count });
    }


    let json_entries = serde_json::to_string_pretty(&entries).unwrap();
    let json_tags = serde_json::to_string_pretty(&tags_vec).unwrap();
    fs::write("../public/blogs/index.json", json_entries.as_bytes()).unwrap();
    fs::write("../public/blogs/tags.json", json_tags.as_bytes()).unwrap();
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

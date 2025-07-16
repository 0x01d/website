use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use glob::glob;
use comrak::{markdown_to_html, ComrakOptions};
use std::env;


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
    path: String,
}

fn main() {
    match env::current_dir() {
        Ok(path) => println!("Current directory: {}", path.display()),
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
    let mut entries = Vec::new();
    for entry in glob("public/blogs/*.md").unwrap() {
        let path = entry.unwrap();
        println!("Parsing: {}", path.display());
        let content = fs::read_to_string(&path).unwrap();

        if let Some((fm, body)) = extract_frontmatter(&content) {
            let html = markdown_to_html(&body, &ComrakOptions::default());

            let html_path = PathBuf::from("public/blogs").join(format!("{}.html", fm.slug));
            fs::write(&html_path, html).unwrap();

            let blog_entry = BlogEntry {
                title: fm.title,
                slug: fm.slug.clone(),
                tags: fm.tags,
                date: fm.date,
                path: format!("/blogs/{}.html", fm.slug),
            };
            entries.push(blog_entry);
        }
    }

    let json = serde_json::to_string_pretty(&entries).unwrap();
    fs::write("public/blogs/index.json", json.as_bytes()).unwrap();
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

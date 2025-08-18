use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use glob::glob;
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style, FontStyle};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
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
    println!("=== Starting Blog Generation ===");
    
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];
    
    println!("Loaded syntax definitions and theme: base16-ocean.dark");
    
    let mut entries = Vec::new();
    let mut tags: HashMap<String, i32> = HashMap::new();
    tags.insert("All".to_string(), 0);
    
    for entry in glob("../blogs/*.md").unwrap() {
        let path = entry.unwrap();
        let slug = path.file_name().unwrap().to_string_lossy().into_owned();
        println!("\nParsing: {}", path.display());
        let content = fs::read_to_string(&path).unwrap();

        if let Some((fm, body)) = extract_frontmatter(&content) {
            println!("  Title: {}", fm.title);
            println!("  Date: {}", fm.date);
            println!("  Tags: {:?}", fm.tags);
            
            // Process the body to add syntax highlighting
            let highlighted_body = process_markdown_code_blocks(&body, &syntax_set, theme);
            
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
            
            fs::write(format!("../public/blogs/{}", slug), highlighted_body.as_bytes()).unwrap();
            println!("  Written to: ../public/blogs/{}", slug);
        } else {
            println!("  Warning: Failed to parse frontmatter");
        }
    }

    // Build tags vector
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

    // Sort entries by date (newest first)
    entries.sort_by(|a, b| b.date.cmp(&a.date));
    
    let json_entries = serde_json::to_string_pretty(&entries).unwrap();
    let json_tags = serde_json::to_string_pretty(&tags_vec).unwrap();
    
    fs::write("../public/blogs/index.json", json_entries.as_bytes()).unwrap();
    fs::write("../public/blogs/tags.json", json_tags.as_bytes()).unwrap();
    
    println!("\n=== Blog Generation Complete ===");
    println!("Processed {} blog posts", entries.len());
    println!("Found {} unique tags", tags_vec.len());
    println!("Output written to ../public/blogs/");
}

fn process_markdown_code_blocks(content: &str, syntax_set: &SyntaxSet, theme: &syntect::highlighting::Theme) -> String {
    let mut result = String::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();
    
    for line in content.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block - highlight and add it
                let highlighted = if code_block_lang.is_empty() {
                    // No language specified, just return the content as-is
                    code_block_content.clone()
                } else {
                    highlight_code(&code_block_content, &code_block_lang, syntax_set, theme)
                };
                
                result.push_str("```");
                if !code_block_lang.is_empty() {
                    result.push_str(&code_block_lang);
                }
                result.push('\n');
                result.push_str(&highlighted);
                result.push_str("```\n");
                
                in_code_block = false;
                code_block_content.clear();
                code_block_lang.clear();
            } else {
                // Start of code block
                in_code_block = true;
                code_block_lang = line[3..].trim().to_string();
            }
        } else if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    
    // Handle unclosed code block
    if in_code_block {
        eprintln!("Warning: Unclosed code block detected");
        result.push_str("```");
        if !code_block_lang.is_empty() {
            result.push_str(&code_block_lang);
        }
        result.push('\n');
        result.push_str(&code_block_content);
        result.push_str("```\n");
    }
    
    result
}

fn highlight_code(code: &str, lang: &str, syntax_set: &SyntaxSet, theme: &syntect::highlighting::Theme) -> String {
    let syntax = syntax_set.find_syntax_by_extension(lang)
        .or_else(|| syntax_set.find_syntax_by_name(lang))
        .or_else(|| {
            // Try common aliases
            match lang {
                "js" => syntax_set.find_syntax_by_extension("javascript"),
                "ts" => syntax_set.find_syntax_by_extension("typescript"),
                "py" => syntax_set.find_syntax_by_extension("python"),
                "rb" => syntax_set.find_syntax_by_extension("ruby"),
                "yml" => syntax_set.find_syntax_by_extension("yaml"),
                _ => None,
            }
        })
        .unwrap_or_else(|| {
            println!("  Warning: Unknown language '{}', using plain text", lang);
            syntax_set.find_syntax_plain_text()
        });
    
    println!("  Highlighting {} code block with syntax: {}", lang, syntax.name);
    
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = String::new();
    let mut token_count = 0;
    
    for line in LinesWithEndings::from(code) {
        if let Ok(ranges) = highlighter.highlight_line(line, syntax_set) {
            for (style, text) in ranges {
                let style_code = style_to_code(style);
                if let Some(code) = style_code {
                    result.push_str(&format!("§{}§{}§/§", code, text));
                    token_count += 1;
                } else {
                    result.push_str(text);
                }
            }
        } else {
            result.push_str(line);
        }
    }
    
    println!("  Added {} style tokens", token_count);
    result
}

fn style_to_code(style: Style) -> Option<char> {
    // More accurate mapping based on base16-ocean.dark theme colors and font styles
    let fg = style.foreground;
    let has_bold = style.font_style.contains(FontStyle::BOLD);
    let has_italic = style.font_style.contains(FontStyle::ITALIC);
    
    // Check for specific color ranges used by base16-ocean.dark
    match (fg.r, fg.g, fg.b) {
        // Comments (gray/muted) - usually italic
        (r, g, b) if has_italic || (r > 100 && r < 180 && g > 100 && g < 180 && b > 100 && b < 180) => Some('c'),
        
        // Keywords (purple/blue) - #b48ead or similar
        (r, g, b) if (r > 170 && r < 190 && g > 100 && g < 150 && b > 160 && b < 180) ||
                     (b > 200 && g < 150 && r < 150) => Some('k'),
        
        // Strings (green) - #a3be8c or similar
        (r, g, b) if (r > 140 && r < 180 && g > 180 && g < 210 && b > 120 && b < 160) ||
                     (g > 170 && r < 170 && b < 160) => Some('s'),
        
        // Numbers/constants (orange/peach) - #d08770 or similar
        (r, g, b) if (r > 200 && g > 120 && g < 160 && b > 100 && b < 130) ||
                     (r > 200 && g > 100 && b < 130) => Some('n'),
        
        // Functions/types (yellow) - #ebcb8b or similar
        (r, g, b) if (r > 220 && g > 190 && b > 130 && b < 160) ||
                     (r > 200 && g > 180 && b < 180) => Some('f'),
        
        // Types/classes (cyan) - #8fbcbb or similar
        (r, g, b) if (r > 120 && r < 160 && g > 180 && g < 210 && b > 180 && b < 210) ||
                     (g > 160 && b > 160 && r < 160) => Some('t'),
        
        // Operators/punctuation (light gray/white)
        (r, g, b) if r > 200 && g > 200 && b > 200 => Some('o'),
        
        // Default for anything else
        _ => None,
    }
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

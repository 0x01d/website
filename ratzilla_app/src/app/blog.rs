use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, ListState, ListItem, List, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState},
    text::Text,
    layout::Rect,
    Frame,
};
use wasm_bindgen_futures::spawn_local;
use gloo_net::http::Request;
use web_sys::console;
use wasm_bindgen::JsValue;

use crate::app::markdown_renderer::MarkdownRenderer;
use crate::app::Msg;

#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub count: u32,
}

#[derive(Clone, Debug)]
pub struct BlogEntry {
    pub title: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub date: String,  // Keep as String for simplicity in client
}

enum Pane {
    Post,
    List
}

pub struct BlogModel {
    loaded: bool,
    mouse_coords: (u16, u16),
    filter_updated: bool,
    pub tag_list: Vec<Tag>,
    pub blog_list: Vec<BlogEntry>,
    pub tag_list_filtered: Vec<Tag>,
    pub blog_list_filtered: Vec<BlogEntry>,
    pub tag_list_state: ListState,
    pub blog_list_state: ListState,
    blog_list_rect: Rect,
    tag_list_rect: Rect,
    pub scrollbar_state: Option<ScrollbarState>,
    pub vertical_scroll: usize,
    active_pane: Pane,
    tx: flume::Sender<Msg>,
    rx: flume::Receiver<Msg>,
    pub loaded_blog: MarkdownRenderer,
}

impl BlogModel {
    pub fn new(tx: flume::Sender<Msg>, rx: flume::Receiver<Msg>) -> Self {
        let tag_list = Vec::new();
        let blog_list = Vec::new();
        let tag_list_filtered = Vec::new();
        let blog_list_filtered = Vec::new();
        let tag_list_state = ListState::default();
        let blog_list_state = ListState::default();
        Self {
            filter_updated: false,
            mouse_coords: (0, 0),
            loaded: false,
            tag_list,
            blog_list,
            tag_list_filtered,
            blog_list_filtered,
            blog_list_rect: Rect::default(),
            tag_list_rect: Rect::default(),
            tag_list_state,
            blog_list_state,
            scrollbar_state: None,
            vertical_scroll: 0,
            active_pane: Pane::Post,
            loaded_blog: MarkdownRenderer::new(),
            tx,
            rx,
        }
    }

    pub fn lazy_load(&mut self) {
        if self.loaded {
            return
        }
        self.fetch_tags();
        self.fetch_index();
        self.loaded = true;
    }

    pub fn update(&mut self, msg: crate::app::Msg) {
        match msg {
            Msg::MouseMove(coords) =>  { 
                self.mouse_coords = coords;
                if self.mouse_coords.0 < self.blog_list_rect.width {
                    self.active_pane = Pane::Post;
                    let new = self.mouse_coords.1.saturating_sub(self.blog_list_rect.y + 1);
                    self.blog_list_state.select(Some(new as usize));
                } else {
                    self.active_pane = Pane::List;
                    let new = self.mouse_coords.1.saturating_sub(self.tag_list_rect.y + 1);
                    self.tag_list_state.select(Some(new as usize));
                }

            }
            Msg::ScrollVert(delta_row) => {
                if delta_row > 0 {
                    self.loaded_blog.scroll_up(delta_row as u16);
                } else {
                    self.loaded_blog.scroll_down(delta_row.abs() as u16);
                }
            }
            Msg::LoadSubPath(ref path) => {
                if let Some(slug) = path.split('/').next() {
                    if slug.is_empty() { return }
                    Self::fetch_blog(slug.to_string(), self.tx.clone());
                    if let Some(tags) = self.blog_list.iter() .find(|e| e.slug == slug) .map(|e| e.tags.as_slice()) {
                        self.filter_tags(&tags.to_vec());
                    }

                }

            }
            Msg::LoadHash(ref hash) => {
                if hash.is_empty() { return }
                let tag = hash.trim_start_matches("#");
                self.filter_blogs(&tag);
            }
            Msg::NavigateBack => { 
                if self.loaded_blog.loaded_blog.is_none() {
                    if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                        let _ = history.push_state_with_url(&JsValue::NULL, "", Some("~"));
                    }
                    self.tx.clone().try_send(Msg::SwitchTo(crate::app::Displays::Splash));
                    return
                } else {
                    if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                        let _ = history.push_state_with_url(&JsValue::NULL, "", Some("/blog"));
                    }

                }
                self.loaded_blog.loaded_blog = None;
            }
            _ => {}
        }
        match self.active_pane {
            Pane::List => {
                let current = self.tag_list_state.selected().unwrap_or(0);
                match msg {
                    Msg::NavigateUp => {
                        let new = current.saturating_sub(1);
                        self.tag_list_state.select(Some(new));
                    }
                    Msg::NavigateDown => {
                        let max = self.tag_list.len().saturating_sub(1);
                        let new = (current + 1).min(max);
                        self.tag_list_state.select(Some(new));
                    }
                    Msg::NavigateLeft => {
                        self.active_pane = Pane::Post;
                    }
                    Msg::NavigateRight => {
                        self.active_pane = Pane::Post;
                    }
                    Msg::Select => {
                        let sel = self.tag_list[current].to_owned();
                        let path = format!("/blog/#{}", &sel.name);
                        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                            let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&path));
                        }
                        self.filter_blogs(&sel.name);
                        self.loaded_blog.loaded_blog = None;
                    }
                    Msg::MouseClick(btn) => {
                        if self.mouse_coords.1 < self.tag_list_rect.y + 1 + self.tag_list.len() as u16 {
                            self.update(Msg::Select);
                        }
                    }
                    _ => {}
                }
            }
            Pane::Post => {
                let current = self.blog_list_state.selected().unwrap_or(0);
                match msg {
                    Msg::NavigateUp => {
                        let new = current.saturating_sub(1);
                        self.blog_list_state.select(Some(new));
                    }
                    Msg::NavigateDown => {
                        let max = self.blog_list.len().saturating_sub(1);
                        let new = (current + 1).min(max);
                        self.blog_list_state.select(Some(new));
                    }
                    Msg::NavigateLeft => {
                        self.active_pane = Pane::List;
                    }
                    Msg::NavigateRight => {
                        self.active_pane = Pane::List;
                    }
                    Msg::Select => {
                        let sel = &self.blog_list[current];
                        let path = format!("/blog/{}", &sel.slug);
                        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                            let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&path));
                        }
                        Self::fetch_blog(sel.slug.to_string(), self.tx.clone());
                        self.filter_tags(&sel.tags.clone());
                    }
                    Msg::MouseClick(btn) => {
                        if self.mouse_coords.1 < self.blog_list_rect.y + 1 + self.blog_list.len() as u16 {
                            self.update(Msg::Select);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn view(&mut self, f: &mut Frame) {
        if self.loaded == false {
            self.lazy_load();
        }
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(f.area());

        self.blog_list_rect = chunks[0];
        self.tag_list_rect = chunks[1];

        let mut items: Vec<ListItem> = Vec::new();

        items.extend(self.tag_list_filtered.iter().map(|tag| {
            ListItem::new(format!("{} ({})", tag.name, tag.count))
        }));        

        let list_active = match self.active_pane {
            Pane::List => true,
            Pane::Post => false,
        };

        let list = List::new(items)
            .block(Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .border_style( Style::default()
                    .fg(if list_active {Color::Yellow} else { Color::Reset })
                )
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        let mut blog_entries: Vec::<ListItem> = Vec::new();

        blog_entries.extend(self.blog_list_filtered.iter().map(|blog| {
            let title_span = Span::styled(&blog.title, Style::default().fg(Color::White));
            let date_span = Span::styled(format!(" - [{}] - ", blog.date), Style::default().fg(Color::White));

            let mut tag_spans: Vec<Span> = Vec::new();
            for (i, tag) in blog.tags.iter().enumerate() {
                tag_spans.push(Span::styled(
                        format!("#{}", tag),
                        Style::default()
                        .fg(Color::Cyan)
                ));

                if i < blog.tags.len() - 1 {
                    tag_spans.push(Span::raw(" "));
                }
            }

            let mut spans = vec![title_span, date_span];
            spans.extend(tag_spans);

            ListItem::new(Line::from(spans))
        }));

        let blog_list = List::new(blog_entries)
            .block(Block::default()
                .title("Posts")
                .borders(Borders::ALL)
                .border_style( Style::default().fg(if !list_active {Color::Yellow} else { Color::Reset }))
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));


        if let Some(ref blog) = &self.loaded_blog.get_content().clone() {
            let blog_paragraph = Paragraph::new(blog.to_owned().to_owned())
                .scroll((self.loaded_blog.vertical_scroll as u16, 0))
                .block(Block::default()
                    .title("Post")
                    .borders(Borders::ALL)
                    .border_style( Style::default().fg(if !list_active {Color::Yellow} else { Color::Reset }))
                );
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(blog_paragraph, chunks[0]);

            if let Some(mut bar) = self.loaded_blog.scrollbar_state {
                f.render_stateful_widget(scrollbar, chunks[0].inner(Margin {vertical:1, horizontal:1}), &mut bar);
            }

        } else {
            f.render_stateful_widget(blog_list, chunks[0], &mut self.blog_list_state);
        }
        f.render_stateful_widget(list, chunks[1], &mut self.tag_list_state);
    }

    pub fn parse_blog_text(&mut self, content: String) {
        self.loaded_blog.parse_blog_text(content);
        if let Some(blog) = &self.loaded_blog.loaded_blog {
            self.scrollbar_state = Some(
                ScrollbarState::new(blog.height())
                .position(self.vertical_scroll)
            );
        }
    }

    pub fn filter_blogs(&mut self, filter_tag: &str) {
        self.blog_list_filtered.clear();

        if filter_tag == "All" {
            self.blog_list_filtered = self.blog_list.clone();
            self.tag_list_filtered = self.tag_list.clone();
        }

        for blog in self.blog_list.iter() {
            for tag in blog.tags.iter() {
                if tag == filter_tag {
                    self.blog_list_filtered.push(blog.to_owned()); 
                }
            }
        }
    }
    pub fn filter_tags(&mut self, blog_tags: &Vec<String>) {
        self.tag_list_filtered.clear();

        for tag in self.tag_list.iter() {
            if blog_tags.contains(&tag.name) || tag.name.contains(&"All".to_string()) {
                self.tag_list_filtered.push(tag.clone());
            }
        }
    } 

    pub fn fetch_tags(&mut self) {
        let tx_clone = self.tx.clone();
        spawn_local(async move{
            let url = "/public/blogs/tags.json";

            match Request::get(url).send().await {
                Ok(response) => {
                    if response.ok() {
                        match response.text().await {
                            Ok(text) => {
                                console::log_1(&format!("Fetched tags: {}", text).into());
                                if let Some(tags) = parse_tags_json(&text) {
                                    let _ = tx_clone.try_send(Msg::UpdateBlogTags(tags));
                                }
                            }
                            Err(err) => {
                                console::error_1(&format!("Failed to read response body: {:?}", err).into());
                            }
                        }
                    } else {
                        console::error_1(&format!("Request failed: {}", response.status()).into());
                    }
                }
                Err(err) => {
                    console::error_1(&format!("Fetch error: {:?}", err).into());
                }
            }
        });
    }

    pub fn fetch_index(&mut self) {
        let tx_clone = self.tx.clone();
        spawn_local(async move{
            let url = "/public/blogs/index.json";

            match Request::get(url).send().await {
                Ok(response) => {
                    if response.ok() {
                        match response.text().await {
                            Ok(text) => {
                                console::log_1(&format!("Fetched index: {}", text).into());
                                if let Some(blogs) = parse_blog_index_json(&text) {
                                    let _ = tx_clone.try_send(Msg::UpdateBlogIndex(blogs));
                                }
                            }
                            Err(err) => {
                                console::error_1(&format!("Failed to read response body: {:?}", err).into());
                            }
                        }
                    } else {
                        console::error_1(&format!("Request failed: {}", response.status()).into());
                    }
                }
                Err(err) => {
                    console::error_1(&format!("Fetch error: {:?}", err).into());
                }
            }
        });
    }

    pub fn fetch_blog(slug: String, tx: flume::Sender<Msg>) {
        spawn_local(async move{
            let url = format!("/public/blogs/{}", slug);

            match Request::get(&url).send().await {
                Ok(response) => {
                    if response.ok() {
                        match response.text().await {
                            Ok(text) => {
                                let _ = tx.try_send(Msg::ParseBlogText(text));
                            }
                            Err(err) => {
                                console::error_1(&format!("Failed to read response body: {:?}", err).into());
                            }
                        }
                    } else {
                        console::error_1(&format!("Request failed: {}", response.status()).into());
                    }
                }
                Err(err) => {
                    console::error_1(&format!("Fetch error: {:?}", err).into());
                }
            }
        });
    }
}

// Simple JSON parsing without serde
fn parse_tags_json(json: &str) -> Option<Vec<Tag>> {
    let mut tags = Vec::new();
    
    // Remove array brackets and split by objects
    let content = json.trim().trim_start_matches('[').trim_end_matches(']');
    
    for obj in content.split("},") {
        let obj = if obj.ends_with('}') { obj } else { &format!("{}}}", obj) };
        
        let mut name = String::new();
        let mut count = 0;
        
        for line in obj.lines() {
            if let Some(n) = extract_json_string(line, "name") {
                name = n;
            } else if let Some(c) = extract_json_number(line, "count") {
                count = c;
            }
        }
        
        if !name.is_empty() {
            tags.push(Tag { name, count });
        }
    }
    
    Some(tags)
}

fn parse_blog_index_json(json: &str) -> Option<Vec<BlogEntry>> {
    let mut entries = Vec::new();
    
    let content = json.trim().trim_start_matches('[').trim_end_matches(']');
    
    for obj in content.split("},") {
        let obj = if obj.ends_with('}') { obj } else { &format!("{}}}", obj) };
        
        let mut title = String::new();
        let mut slug = String::new();
        let mut tags = Vec::new();
        let mut date = String::new();
        let mut lines = obj.lines();

        while let Some(line) = lines.next() {
            if let Some(t) = extract_json_string(line, "title") {
                title = t;
            } else if let Some(s) = extract_json_string(line, "slug") {
                slug = s;
            } else if let Some(d) = extract_json_string(line, "date") {
                date = d;
            } else if line.contains("\"tags\"") {
                // Now gather tags until we hit a closing ']'
                while let Some(tag_line) = lines.next() {
                    if tag_line.contains("]") {
                        break;
                    }

                    // Each tag line looks like:  "wasm", or "rust",
                    // so strip whitespace, commas, and quotes
                    let cleaned = tag_line
                        .trim()
                        .trim_end_matches(',')
                        .trim_matches('"');

                    if !cleaned.is_empty() {
                        tags.push(cleaned.to_string());
                    }
                }
            }
        }        
        if !title.is_empty() && !slug.is_empty() {
            entries.push(BlogEntry { title, slug, tags, date });
        }
    }

    Some(entries)
}

fn extract_json_string(line: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    if line.contains(&pattern) {
        if let Some(start) = line.find(": \"") {
            let value_start = start + 3;
            if let Some(end) = line[value_start..].find('"') {
                return Some(line[value_start..value_start + end].to_string());
            }
        }
    }
    None
}

fn extract_json_number(line: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\"", key);
    if line.contains(&pattern) {
        if let Some(start) = line.find(": ") {
            let value_start = start + 2;
            let value_str = line[value_start..].trim().trim_end_matches(',');
            return value_str.parse().ok();
        }
    }
    None
}

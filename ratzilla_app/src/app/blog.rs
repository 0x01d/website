use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, ListState, ListItem, List, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState},
    text::Text,
    layout::Rect,
    Frame,
};
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use chrono::NaiveDate;
use web_sys::console;
use wasm_bindgen::JsValue;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style as SynStyle};
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;

use crate::app::Msg;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    name: String,
    count: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BlogEntry {
    title: String,
    slug: String,
    tags: Vec<String>,
    date: NaiveDate,
}

enum Pane {
    Post,
    List
}

pub struct BlogModel {
    loaded: bool,
    filter_updated: bool,
    pub tag_list: Vec<Tag>,
    pub blog_list: Vec<BlogEntry>,
    tag_list_filtered: Vec<Tag>,
    pub blog_list_filtered: Vec<BlogEntry>,
    pub tag_list_state: ListState,
    pub blog_list_state: ListState,
    pub scrollbar_state: Option<ScrollbarState>,
    pub vertical_scroll: usize,
    active_pane: Pane,
    tx: flume::Sender<Msg>,
    rx: flume::Receiver<Msg>,
    pub loaded_blog: Option<Text<'static>>,
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
            loaded: false,
            tag_list,
            blog_list,
            tag_list_filtered,
            blog_list_filtered,
            tag_list_state,
            blog_list_state,
            scrollbar_state: None,
            vertical_scroll: 0,
            active_pane: Pane::Post,
            loaded_blog: None,
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
            Msg::LoadSubPath(ref path) => {
                if let Some(slug) = path.split('/').next() {
                    Self::fetch_blog(slug.to_string(), self.tx.clone());
                }

            }
            Msg::LoadHash(ref hash) => {
                console::log_1(&hash.into());
                let tag = hash.trim_start_matches("#");
                self.filter_blogs(&tag);
            }
            Msg::NavigateBack => { 
                if self.loaded_blog.is_none() {
                    self.tx.clone().try_send(Msg::SwitchTo(crate::app::Displays::Splash));
                    return
                }
                self.loaded_blog = None;
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
                        self.loaded_blog = None;
                        //Self::fetch_blog(sel.slug.to_string(), self.tx.clone())
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

            // Tags with background, spaces without
            let mut tag_spans: Vec<Span> = Vec::new();
            for (i, tag) in blog.tags.iter().enumerate() {
                tag_spans.push(Span::styled(
                        format!("#{}", tag),
                        Style::default()
                        .fg(Color::Cyan)
                        //.bg(),
                ));

                // Add raw unstyled space *after* each tag except the last
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


        if let Some(blog) = &self.loaded_blog {
            let blog_paragraph = Paragraph::new(blog.to_owned())
                .scroll((self.vertical_scroll as u16, 0))
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

            if let Some(mut bar) = self.scrollbar_state {
                f.render_stateful_widget(scrollbar, chunks[0].inner(Margin {vertical:1, horizontal:1}), &mut bar);
            }

        } else {
            f.render_stateful_widget(blog_list, chunks[0], &mut self.blog_list_state);
        }
        f.render_stateful_widget(list, chunks[1], &mut self.tag_list_state);
    }

    pub fn parse_blog_text(&mut self, content: String) {
        use pulldown_cmark::{Event, Parser, TextMergeStream};

        let iterator = TextMergeStream::new(Parser::new(&content));

        let mut content_styled: Text = Text::default();

        for event in iterator {
            match event {
                Event::Text(text) => content_styled.extend(Text::from(text.to_string())),
                _ => {}
            }
        }       

        self.loaded_blog = Some(content_styled.clone());
        self.scrollbar_state = Some(ScrollbarState::new(content_styled.height()).position(self.vertical_scroll));
    }

    pub fn filter_blogs(&mut self, filter_tag: &str) {
        self.blog_list_filtered.clear();

        if filter_tag == "All" { self.blog_list_filtered = self.blog_list.clone();}

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
                                let tag_list: Option<Vec<Tag>> = serde_json::from_str(&text.to_string()).ok();
                                if let Some(tags) = tag_list {
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
                                let blog_list: Option<Vec<BlogEntry>> = serde_json::from_str(&text.to_string()).ok();
                                if let Some(blogs) = blog_list {
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


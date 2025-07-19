use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, ListState, ListItem, List},
    layout::Rect,
    Frame,
};
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use web_sys::console;

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
    date: String,
}

enum Pane {
    Post,
    List
}

pub struct BlogModel {
    loaded: bool,
    pub tag_list: Vec<Tag>,
    pub blog_list: Vec<BlogEntry>,
    pub tag_list_state: ListState,
    pub blog_list_state: ListState,
    active_pane: Pane,
    tx: flume::Sender<Msg>,
    rx: flume::Receiver<Msg>,
}

impl BlogModel {
    pub fn new(tx: flume::Sender<Msg>, rx: flume::Receiver<Msg>) -> Self {
        let tag_list = Vec::new();
        let blog_list = Vec::new();
        let tag_list_state = ListState::default();
        let blog_list_state = ListState::default();
        Self {
            loaded: false,
            tag_list,
            blog_list,
            tag_list_state,
            blog_list_state,
            active_pane: Pane::Post,
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
        match self.active_pane {
            Pane::List => {
                console::log_1(&format!("Tag window selected").into());
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
                    Msg::Select => {

                        //let sel = self.menu_items[current];

                    }
                    Msg::NavigateLeft => {
                        self.active_pane = Pane::Post;
                    }
                    Msg::NavigateRight => {
                        self.active_pane = Pane::Post;
                    }
                    _ => {}
                }
            }
            Pane::Post => {
                console::log_1(&format!("Blog window selected").into());
                let current = self.blog_list_state.selected().unwrap_or(0);
                match msg {
                    Msg::NavigateUp => {
                        let new = current.saturating_sub(1);
                        self.blog_list_state.select(Some(new));
                        console::log_1(&format!("{}", new).into());
                    }
                    Msg::NavigateDown => {
                        let max = self.blog_list.len().saturating_sub(1);
                        let new = (current + 1).min(max);
                        console::log_1(&format!("{}", new).into());
                        self.blog_list_state.select(Some(new));
                    }
                    Msg::Select => {
                        //let sel = self.menu_items[current];

                    }
                    Msg::NavigateLeft => {
                        self.active_pane = Pane::List;
                    }
                    Msg::NavigateRight => {
                        self.active_pane = Pane::List;
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

        items.extend(self.tag_list.iter().map(|tag| {
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
                .border_style( Style::default().fg(if list_active {Color::Yellow} else { Color::Reset }))
            )
            .highlight_style(Style::default().bg(Color::Yellow));

        let mut blog_entries: Vec::<ListItem> = Vec::new();

        blog_entries.extend(self.blog_list.iter().map(|blog| {
            ListItem::new(format!("{} [{}]", blog.title, blog.date))
        })); 

        let blog_list = List::new(blog_entries)
            .block(Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .border_style( Style::default().fg(if !list_active {Color::Yellow} else { Color::Reset }))
            )
            .highlight_style(Style::default().bg(Color::Yellow));

        f.render_stateful_widget(blog_list, chunks[0], &mut self.blog_list_state);
        f.render_stateful_widget(list, chunks[1], &mut self.tag_list_state);
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
                                if let Some(mut tags) = tag_list {
                                    let count: u32 = tags.iter().map(|tag| tag.count).sum();
                                    tags.insert(0, Tag {name: "All".to_string(), count });
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
                                if let Some(mut blogs) = blog_list {
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
}

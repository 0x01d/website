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

enum Pane {
    Post,
    List
}

pub struct BlogModel {
    loaded: bool,
    pub tag_list: Vec<Tag>,
    tag_list_state: ListState,
    active_pane: Pane,
    tx: flume::Sender<Msg>,
    rx: flume::Receiver<Msg>,
}

impl BlogModel {
    pub fn new(tx: flume::Sender<Msg>, rx: flume::Receiver<Msg>) -> Self {
        let tag_list = Vec::new();
        let mut tag_list_state = ListState::default();
        tag_list_state.select(Some(0));
        Self {
            loaded: false,
            tag_list,
            tag_list_state,
            active_pane: Pane::List,
            tx,
            rx,
        }
    }

    pub fn lazy_load(&mut self) {
        if self.loaded {
            return
        }
        self.fetch_tags();
        self.loaded = true;
    }
    pub fn update(&mut self, msg: crate::app::Msg) {
        let mut current_li: usize;
        let mut current_list_state: &mut ListState;

        match self.active_pane {
            Pane::List => {
                current_li = self.tag_list_state.selected().unwrap_or(0);
                current_list_state = &mut self.tag_list_state;
            }
            Pane::Post => {
                current_li = self.tag_list_state.selected().unwrap_or(0);
                current_list_state = &mut self.tag_list_state;
            }
        }
        match msg {
            Msg::NavigateUp => {
                let new = current_li.saturating_sub(1);
                current_list_state.select(Some(new));
            }
            Msg::NavigateDown => {
                let max = self.tag_list.len().saturating_sub(1);
                let new = (current_li + 1).min(max);
                current_list_state.select(Some(new));
            }
            Msg::Select => {
                //let sel = self.menu_items[current];

            }
            _ => {}
        }
    }

    pub fn view(&mut self, f: &mut Frame) {
        if self.loaded == false {
            self.lazy_load();
        }
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(f.area());

        let mut items: Vec<ListItem> = Vec::new();

        // Other items: Each tag with its count
        items.extend(self.tag_list.iter().map(|tag| {
            ListItem::new(format!("{} ({})", tag.name, tag.count))
        }));        

        let list = List::new(items)
            .block(Block::default().title("Tags").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, chunks[0], &mut self.tag_list_state);
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
}

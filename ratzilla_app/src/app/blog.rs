use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, ListState, ListItem, List},
    layout::Rect,
    Frame,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Request, RequestInit, RequestMode, Response};
use js_sys::Promise;
use wasm_bindgen::JsCast;

enum Pane {
    Post,
    List
}

pub struct BlogModel {
    tag_list: Option<HashMap<&str, u32>>,
    tag_list_state: ListState,
    active_pane: Pane,
}

impl BlogModel {
    pub fn new() -> Self {
        let tag_list = None;
        let mut tag_list_state = ListState::default();
        tag_list_state.select(Some(0));
        Self {
            tag_list,
            tag_list_state,
            active_pane: Pane::List
        }
    }

    pub fn update(&mut self, _msg: crate::app::Msg) {
        // handle blog-specific messages here
    }

    pub fn view(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(f.area());
        let tags = if let Some(tags) = self.tag_list.clone() {
            tags
        } else {
            Vec::new()
        };

        let total_count: u32 = tags.iter().map(|tag| tag.count).sum();

        let mut items: Vec<ListItem> = Vec::new();

        // First item: "All" with total count
        items.push(ListItem::new(format!("All ({})", total_count)));

        // Other items: Each tag with its count
        items.extend(tags.iter().map(|tag| {
            ListItem::new(format!("{} ({})", tag.name, tag.count))
        }));        

        let list = List::new(items)
            .block(Block::default().title("Tags").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, chunks[0], &mut self.tag_list_state);
    }

    pub fn fetch_tags(&self) -> Vec<Tag> {
        spawn_local(async {
            let url = "/public/blogs/tags.json";

            match Request::get(url).send().await {
                Ok(response) => {
                    if response.ok() {
                        match response.text().await {
                            Ok(text) => {
                                console::log_1(&format!("Fetched tags: {}", text).into());
                                // Optionally parse JSON here
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

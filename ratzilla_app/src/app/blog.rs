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


struct Tag {
    name: String,
    count: u32,
}

enum Pane {
    Post,
    List
}

pub struct BlogModel {
    tag_list: Vec<Tag>,
    tag_list_state: ListState,
    active_pane: Pane,
}

impl BlogModel {
    pub fn new() -> Self {
        let tag_list = Vec::new();
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

        let total_count: u32 = self.tag_list.iter().map(|tag| tag.count).sum();

        let mut items: Vec<ListItem> = Vec::new();

        // First item: "All" with total count
        items.push(ListItem::new(format!("All ({})", total_count)));

        // Other items: Each tag with its count
        items.extend(self.tag_list.iter().map(|tag| {
            ListItem::new(format!("{} ({})", tag.name, tag.count))
        }));        

        let list = List::new(items)
            .block(Block::default().title("Tags").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, chunks[0], &mut self.tag_list_state);
    }

    pub fn fetch_tags_and_index(&self) {
        spawn_local(async move {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            //let request = Request::new_with_str_and_init("



        });

    }
}

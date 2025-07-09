use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct BlogModel;

impl BlogModel {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _msg: crate::app::Msg) {
        // handle blog-specific messages here
    }

    pub fn view(&mut self, f: &mut Frame, area: Rect) {
        let p = Paragraph::new("Welcome to the blog!")
            .block(Block::default().title("Blog").borders(Borders::ALL));
        f.render_widget(p, area);
    }
}

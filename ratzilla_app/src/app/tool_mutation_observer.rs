use wasm_bindgen_futures::spawn_local;
use ratatui::{
    prelude::*,
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use crate::app::tools::window_scanner::DeepExtensionScanner;
use crate::app::Msg;

pub struct MutationObserverModel {
    output: Text<'static>,
    deep_scanner: DeepExtensionScanner,

}

impl MutationObserverModel {
    pub fn new() -> MutationObserverModel {
        Self {
            output: Text::default(),
            deep_scanner: DeepExtensionScanner::new(),
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::GetReport => {
                let report = self.deep_scanner.generate_report();
                for lin in report {
                    self.output.push_line(lin);
                }
            }
            _ => {}
        }

    }

    pub fn view(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),  
                Constraint::Percentage(50), 
            ])
            .split(f.area());
        let p = Paragraph::new(self.output.clone())
            .block(Block::default().title("Deep Scan").borders(Borders::ALL));

        f.render_widget(p, chunks[0]);
    }
}

use ratatui::{
    prelude::*,
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use crate::app::tools::window_scanner::DeepExtensionScanner;
use crate::app::Msg;

struct MutationObserverModel {
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

    }

    pub fn view(&self, f: &mut Frame) {

    }
}

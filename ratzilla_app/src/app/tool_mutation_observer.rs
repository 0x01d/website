use wasm_bindgen_futures::spawn_local;
use ratatui::{
    prelude::*,
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};
use crate::app::tools::window_scanner::DeepExtensionScanner;
use crate::app::Msg;

pub struct MutationObserverModel {
    output: Text<'static>,
    deep_scanner: DeepExtensionScanner,
    vertical_scroll: usize,
    height_window_scan: u16,

}

impl MutationObserverModel {
    pub fn new() -> MutationObserverModel {
        Self {
            output: Text::default(),
            deep_scanner: DeepExtensionScanner::new(),
            vertical_scroll: 0,
            height_window_scan: 0,
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
            Msg::NavigateDown => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(self.height_window_scan as usize);
            }
            Msg::NavigateUp => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(self.height_window_scan as usize);
            }
            _ => {}
        }

    }

    pub fn view(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80),  
                Constraint::Percentage(20), 
            ])
            .split(f.area());
        self.height_window_scan = chunks[0].height;
        let p = Paragraph::new(self.output.clone())
            .block(Block::default().title("Deep Scan").borders(Borders::ALL))
            .scroll((self.vertical_scroll as u16, 0));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(self.output.height())
            .position(self.vertical_scroll)
            .viewport_content_length(chunks[0].height as usize);
        f.render_widget(p, chunks[0]);
        f.render_stateful_widget(
            scrollbar,
            chunks[0].inner(Margin {
                // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

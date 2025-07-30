use wasm_bindgen_futures::spawn_local;
use ratatui::{
    prelude::*,
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};
use crate::app::tools::window_scanner::{
    DeepExtensionScanner, ExtensionMonitor
};
use crate::app::Msg;

pub struct MutationObserverModel {
    output_deep: Text<'static>,
    output_notif: Text<'static>,
    deep_scanner: DeepExtensionScanner,
    extension_monitor: ExtensionMonitor,
    vertical_scroll: usize,
    vertical_scroll_obs: usize,
    height_window_scan: u16,
    tx: flume::Sender<Msg>,

}

impl MutationObserverModel {
    pub fn new(tx: flume::Sender<Msg>) -> MutationObserverModel {
        Self {
            output_deep: Text::default(),
            output_notif: Text::default(),
            deep_scanner: DeepExtensionScanner::new(),
            extension_monitor: ExtensionMonitor::new(),
            vertical_scroll: 0,
            vertical_scroll_obs: 0,
            height_window_scan: 0,
            tx 
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::GetReport => {
                let report = self.deep_scanner.generate_report();
                for lin in report {
                    self.output_deep.push_line(lin);
                }
            }
            Msg::StartScan => {
                let _ = self.extension_monitor.start_monitoring(self.tx.clone());
            }
            Msg::ReturnMutationResult(ref notif) => {
                for e in notif.0.iter() {
                    let line = Line::from(format!("{}: {}", e.0, e.1));
                    self.output_notif.lines.insert(0, line);
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

        let p = Paragraph::new(self.output_deep.clone())
            .block(Block::default().title("Deep Scan").borders(Borders::ALL))
            .scroll((self.vertical_scroll as u16, 0));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(self.output_deep.height())
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

        //=====================================================================
        let p_obs = Paragraph::new(self.output_notif.clone())
            .block(Block::default().title("Mutation Observer").borders(Borders::ALL))
            .scroll((self.vertical_scroll_obs as u16, 0));

        let scrollbar_obs = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state_obs = ScrollbarState::new(self.output_notif.height())
            .position(self.vertical_scroll)
            .viewport_content_length(chunks[0].height as usize);

        f.render_widget(p_obs, chunks[1]);
        f.render_stateful_widget(
            scrollbar_obs,
            chunks[1].inner(Margin {
                // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state_obs,
        );

    }
}

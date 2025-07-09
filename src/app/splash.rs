use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use crate::app::displays::Displays;
use crate::app::Msg;

pub struct SplashModel {
    menu_items: Vec<Displays>,
    list_state: ListState,
}

impl SplashModel {
    pub fn new() -> Self {
        let items = Displays::all_visible();
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            menu_items: items,
            list_state: state,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Option<Msg> {
        let current = self.list_state.selected().unwrap_or(0);
        match msg {
            Msg::NavigateUp => {
                let new = current.saturating_sub(1);
                self.list_state.select(Some(new));
                None
            }
            Msg::NavigateDown => {
                let max = self.menu_items.len().saturating_sub(1);
                let new = (current + 1).min(max);
                self.list_state.select(Some(new));
                None
            }
            Msg::Select => {
                let sel = self.menu_items[current];
                return Some(Msg::SwitchTo(sel.into()));
            }
            _ => None,
        }
    }

    pub fn view(&mut self, f: &mut Frame) {
        let items: Vec<ListItem> = self
            .menu_items
            .iter()
            .map(|&d| ListItem::new(d.label()))
            .collect();
        let area = ratatui::layout::Rect::new(0,0,80,100);

        let list = List::new(items)
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, area, &mut self.list_state);
    }
}

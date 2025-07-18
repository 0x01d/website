use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use crate::app::displays::Displays;
use crate::app::Msg;
use crate::app::tui_helpers::centered_rect;
use crate::app::intro::IntroModel;

pub struct SplashModel {
    intro: IntroModel,
    menu_items: Vec<Displays>,
    list_state: ListState,
}

impl SplashModel {
    pub fn new() -> Self {
        let items = Displays::all_visible();
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            intro: IntroModel::new(),
            menu_items: items,
            list_state: state,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Vec<Msg> {
        let current = self.list_state.selected().unwrap_or(0);
        let mut msg_list: Vec<Msg> = Vec::new();
        match msg {
            Msg::NavigateUp => {
                let new = current.saturating_sub(1);
                self.list_state.select(Some(new));
                msg_list
            }
            Msg::NavigateDown => {
                let max = self.menu_items.len().saturating_sub(1);
                let new = (current + 1).min(max);
                self.list_state.select(Some(new));
                msg_list
            }
            Msg::Select => {
                let sel = self.menu_items[current];
                msg_list.push(Msg::SwitchTo(sel.into()));
                msg_list.push(Msg::PushStateFromDisplay(sel.into()));

                msg_list
            }
            _ => msg_list
        }
    }

    pub fn view(&mut self, f: &mut Frame) {
        let items: Vec<ListItem> = self
            .menu_items
            .iter()
            .map(|&d| ListItem::new(d.label()))
            .collect();

        let mut menu_height: u16 = items.len().try_into().unwrap();
        menu_height += 2;
        // Split the inner area into three vertical chunks:
        // [ ASCII Art + Title ] [ Spacer ] [ Menu ]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(21),  // ASCII art + title
                Constraint::Length(1),     // Spacer
                Constraint::Length(menu_height),  // Menu height
            ])
            .split(f.area());        
        self.intro.view(f, chunks[0]);
        let menu_area = centered_rect(23, 100, chunks[2]);

        let list = List::new(items)
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, menu_area, &mut self.list_state);
    }
}

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use crate::app::displays::Displays;
use crate::app::Msg;
use crate::app::tui_helpers::centered_rect;
use crate::app::intro::IntroModel;

pub struct SplashModel {
    intro: IntroModel,
    menu_items: Vec<Displays>,
    list_state: ListState,
    mouse_coords: (u16, u16),
    menu_area: Rect,
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
            mouse_coords: (0,0),
            menu_area: Rect::default(),
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
            Msg::MouseMove(coords) =>  { 
                self.mouse_coords = coords;
                if self.mouse_coords.1 > self.menu_area.y && self.mouse_coords.1 <= (self.menu_area.y + self.menu_items.len() as u16){
                    if self.mouse_coords.0 > self.menu_area.x && self.mouse_coords.0 < (self.menu_area.x + self.menu_area.width) {
                        let new = self.mouse_coords.1.saturating_sub(self.menu_area.y + 1);
                        self.list_state.select(Some(new as usize));
                    }
                }
                    
                msg_list
            }
            Msg::MouseClick(coords) =>  { 
                if self.mouse_coords.1 > self.menu_area.y && self.mouse_coords.1 <= (self.menu_area.y + self.menu_items.len() as u16){
                    if self.mouse_coords.0 > self.menu_area.x && self.mouse_coords.0 < (self.menu_area.x + self.menu_area.width) {
                        let new = self.mouse_coords.1.saturating_sub(self.menu_area.y + 1);
                        self.list_state.select(Some(new as usize));
                        let sel = self.menu_items[current];
                        msg_list.push(Msg::SwitchTo(sel.into()));
                        msg_list.push(Msg::PushStateFromDisplay(sel.into()));
                    }
                }

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
                Constraint::Length(2),
            ])
            .split(f.area());        
        self.intro.view(f, chunks[0]);
        self.menu_area = centered_rect(23, 100, chunks[2]);

        let list = List::new(items)
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Yellow));

        /* let p: Paragraph = Paragraph::new(
            vec![
            Line::from(format!("x:{},y:{}", self.mouse_coords.0, self.mouse_coords.1)),
            Line::from(format!("{},{},{},{}", self.menu_area.x, self.menu_area.y, self.menu_area.width, self.menu_area.height))
            ]
        );
        f.render_widget(p, chunks[3]); */

        f.render_stateful_widget(list, self.menu_area, &mut self.list_state);
    }
}

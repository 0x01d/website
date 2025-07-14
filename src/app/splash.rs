use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use crate::app::displays::Displays;
use crate::app::Msg;
use crate::app::tui_helpers::centered_rect;

static ASCII: &'static str = 
"
                                 _____          
        ______                  /\\   \\        
       |::|   |                /::\\   \\       
       |::|   |               /::::\\   \\      
       |::|   |              /::::::\\   \\     
      |::|   |             /:::/\\:::\\   \\    
       |::|   |            /:::/  \\:::\\   \\   
       |::|   |           /:::/    \\:::\\   \\  
       |::|   |          /:::/     /\\:::\\   \\ 
 ______|::|___|___ ____ /:::/     /  \\:::\\___\\
|:::::::::::::::::|    /:::/____ /    \\:::|    |
|:::::::::::::::::|____\\:::\\   \\    /:::|____|
";
/*
 ~~~~~~|::|~~~|~~~      \:::\    \   /:::/    / 
       |::|   |          \:::\    \ /:::/    /  
       |::|   |           \:::\    /:::/    /   
       |::|   |            \:::\  /:::/    /    
       |::|   |             \:::\/:::/    /     
       |::|   |              \::::::/    /      
       |::|   |               \::::/    /       
       |::|___|                \::/____/        
        ~~                      ~~              
";*/
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

        let full_area = centered_rect(60, 40, f.area());

        // Draw a border around the full area
        let bordered_block = Block::default()
            .title("Main Area")
            .borders(Borders::ALL);
        f.render_widget(bordered_block.clone(), full_area);

        let inner_area = bordered_block.inner(full_area);
        let menu_height: u16 = items.len().try_into().unwrap();
        // Split the inner area into three vertical chunks:
        // [ ASCII Art + Title ] [ Spacer ] [ Menu ]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // ASCII art + title
                Constraint::Min(1),     // Spacer
                Constraint::Length(menu_height),  // Menu height
            ])
            .split(inner_area);        

        // Render ASCII art and title
        let ascii_art = Paragraph::new("")
            .block(Block::default())
            .alignment(Alignment::Center);
        f.render_widget(ascii_art, chunks[0]);

        let menu_area = centered_rect(60, 100, chunks[2]);

        let list = List::new(items)
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, menu_area, &mut self.list_state);
    }
}

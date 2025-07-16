use ratzilla::{
    event::{KeyCode, KeyEvent},
};
use ratatui::Frame;

mod intro;
mod displays;
mod splash;
mod blog;
mod tui_helpers;

use displays::Displays;
use splash::SplashModel;
use blog::BlogModel;
use intro::IntroModel;

pub enum Msg {
    NavigateUp,
    NavigateDown,
    Select,
    SwitchTo(Displays),
}

pub struct App {
    pub current: Displays,
    splash: SplashModel,
    blog: BlogModel,
    intro: IntroModel,
}

impl App {
    pub fn new() -> Self {
        Self {
            current: Displays::Splash,
            intro: IntroModel::new(),
            splash: SplashModel::new(),
            blog: BlogModel::new(),
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SwitchTo(s) => {
                self.current = s;
                match s {
                    Displays::Blog => {
                        self.blog.fetch_tags_and_index();
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        match self.current {
            Displays::Splash => {
                if let Some(m) = self.splash.update(msg) {
                    self.update(m);
                }
            }
            Displays::Blog => {
                match msg {
                    _ => self.blog.update(msg),
                }
            }
            _ => {}
        }
    }
/*
    pub fn view<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        match self.current {
            AppState::Splash => self.splash.view(f, area),
            AppState::Blog => self.blog.view(f, area),
        }
    }
*/
    pub fn render(&mut self, frame: &mut Frame) {
        match self.current {
            Displays::Splash => self.splash.view(frame),
            Displays::Blog => self.blog.view(frame),
            _ => {}
        } 
    }

    pub fn handle_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.update(Msg::NavigateUp),
            KeyCode::Down => self.update(Msg::NavigateDown),
            KeyCode::Enter => self.update(Msg::Select),
            KeyCode::Esc => self.update(Msg::SwitchTo(Displays::Splash)),
            _ =>  {}
        };
    }

}

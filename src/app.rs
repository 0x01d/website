use ratzilla::{
    event::{KeyCode, KeyEvent},
};
use ratatui::Frame;

mod displays;
mod splash;
mod blog;

use displays::Displays;
use splash::SplashModel;
use blog::BlogModel;

pub enum AppState {
    Splash,
    Blog,
    // Add more states e.g. Tools, About...
}

pub enum Msg {
    NavigateUp,
    NavigateDown,
    Select,
    SwitchTo(AppState),
}

impl From<Displays> for AppState {
    fn from(d: Displays) -> Self {
        match d {
            Displays::Blog => AppState::Blog,
            // map others here...
            _ => AppState::Splash,
        }
    }
}

pub struct App {
    pub current: AppState,
    splash: SplashModel,
    blog: BlogModel,
}

impl App {
    pub fn new() -> Self {
        Self {
            current: AppState::Splash,
            splash: SplashModel::new(),
            blog: BlogModel::new(),
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match self.current {
            AppState::Splash => {
                if let Some(m) = self.splash.update(msg) {
                    self.update(m);
                }
            }
            AppState::Blog => {
                match msg {
                    Msg::SwitchTo(s) => self.current = s,
                    _ => self.blog.update(msg),
                }
            }
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
            AppState::Splash => self.splash.view(frame),
            //AppState::Blog => self.blog.view(frame)
            _ => {}
        } 
    }

    pub fn handle_events(&self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => {} ,
            KeyCode::Right => {},
            _ => {}
        }
    }

}

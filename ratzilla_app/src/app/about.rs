use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use crate::app::markdown_renderer::MarkdownRenderer;
static MARKDOWN_CONTENT: &str = "
# About


Personal website/blog of Ruben Käller, a full stack software developer with an affinity for Linux and AI. 

The website is a Work In Progress.
";
pub struct AboutModel {
    markdown_renderer: MarkdownRenderer
}

impl AboutModel {
    pub fn new() -> AboutModel {
        let mut markdown_renderer = MarkdownRenderer::new();
        markdown_renderer.parse_blog_text(MARKDOWN_CONTENT.to_string());
        Self {
            markdown_renderer
        }
    }

    pub fn view(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(12), Constraint::Percentage(76), Constraint::Percentage(12)].as_ref())
            .split(f.area());

        if let Some(blog) = &self.markdown_renderer.loaded_blog {
            let blog_paragraph = Paragraph::new(blog.to_owned())
                .scroll((self.markdown_renderer.vertical_scroll as u16, 0))
                .block(Block::default()
                    .title("rbn.dev")
                    .borders(Borders::ALL)
                    .border_style( Color::Yellow)
                );
            f.render_widget(blog_paragraph, chunks[1]);
        }
    }

}

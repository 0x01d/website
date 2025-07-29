use pulldown_cmark::{Event, Parser, Tag, TagEnd, CodeBlockKind, TextMergeStream};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{ThemeSet, Style as SyntectStyle},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

pub struct MarkdownRenderer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    pub loaded_blog: Option<Text<'static>>,
    pub scrollbar_state: Option<ratatui::widgets::ScrollbarState>,
    pub vertical_scroll: usize,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            loaded_blog: None,
            scrollbar_state: None,
            vertical_scroll: 0,
        }
    }

    pub fn parse_blog_text(&mut self, content: String) {
        let iterator = TextMergeStream::new(Parser::new(&content));
        let mut content_styled: Text<'static> = Text::default();
        let mut current_line: Vec<Span<'static>> = Vec::new();
        let mut style_stack: Vec<Style> = vec![Style::default()];
        let mut in_code_block = false;
        let mut code_block_content = String::new();
        let mut code_block_lang: Option<String> = None;
        let mut list_depth: usize = 0;
        let mut in_blockquote = false;

        for event in iterator {
            match event {
                Event::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(&text);
                    } else {
                        let current_style = *style_stack.last().unwrap_or(&Style::default());
                        current_line.push(Span::styled(text.to_string(), current_style));
                    }
                }

                Event::Code(code_content) => {
                    let inline_code_style = Style::default()
                        .bg(Color::Rgb(40, 40, 40))
                        .fg(Color::Rgb(255, 182, 193))
                        .add_modifier(Modifier::BOLD);
                    current_line.push(Span::styled(code_content.to_string(), inline_code_style));
                }

                Event::Start(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            if in_blockquote {
                                current_line.push(Span::styled("▎ ", Style::default().fg(Color::Gray)));
                            }
                        }

                        Tag::Heading { level, .. } => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            
                            let heading_style = match level {
                                pulldown_cmark::HeadingLevel::H1 => Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                                pulldown_cmark::HeadingLevel::H2 => Style::default()
                                    .fg(Color::Blue)
                                    .add_modifier(Modifier::BOLD),
                                pulldown_cmark::HeadingLevel::H3 => Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                                _ => Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            };
                            
                            // Add heading prefix
                            let prefix = "#".repeat(level as usize);
                            current_line.push(Span::styled(format!("{} ", prefix), heading_style));
                            style_stack.push(heading_style);
                        }

                        Tag::CodeBlock(kind) => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            in_code_block = true;
                            code_block_content.clear();
                            
                            code_block_lang = match kind {
                                CodeBlockKind::Fenced(lang) => {
                                    if lang.is_empty() {
                                        None
                                    } else {
                                        Some(lang.to_string())
                                    }
                                }
                                CodeBlockKind::Indented => None,
                            };
                        }

                        Tag::BlockQuote(_) => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            in_blockquote = true;
                            let quote_style = Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC);
                            style_stack.push(quote_style);
                        }

                        Tag::List(_) => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            list_depth += 1;
                        }

                        Tag::Item => {
                            let indent = "  ".repeat(list_depth.saturating_sub(1));
                            current_line.push(Span::styled(format!("{}• ", indent), Style::default().fg(Color::Yellow)));
                        }

                        Tag::Emphasis => {
                            let emphasis_style = style_stack.last().unwrap_or(&Style::default())
                                .add_modifier(Modifier::ITALIC);
                            style_stack.push(emphasis_style);
                        }

                        Tag::Strong => {
                            let strong_style = style_stack.last().unwrap_or(&Style::default())
                                .add_modifier(Modifier::BOLD);
                            style_stack.push(strong_style);
                        }

                        Tag::Strikethrough => {
                            let strike_style = style_stack.last().unwrap_or(&Style::default())
                                .add_modifier(Modifier::CROSSED_OUT);
                            style_stack.push(strike_style);
                        }

                        Tag::Link { dest_url, .. } => {
                            let link_style = Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::UNDERLINED);
                            style_stack.push(link_style);
                        }

                        _ => {}
                    }
                }

                Event::End(tag) => {
                    match tag {
                        TagEnd::Paragraph => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            content_styled.extend(Text::from(Line::from(""))); // Add blank line after paragraph
                        }

                        TagEnd::Heading { .. } => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            content_styled.extend(Text::from(Line::from(""))); // Add blank line after heading
                            style_stack.pop();
                        }

                        TagEnd::CodeBlock => {
                            if in_code_block {
                                self.render_code_block(&code_block_content, &code_block_lang, &mut content_styled);
                                in_code_block = false;
                                content_styled.extend(Text::from(Line::from(""))); // Add blank line after code block
                            }
                        }

                        TagEnd::BlockQuote(_) => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            in_blockquote = false;
                            style_stack.pop();
                            content_styled.extend(Text::from(Line::from(""))); // Add blank line after blockquote
                        }

                        TagEnd::List(_) => {
                            list_depth = list_depth.saturating_sub(1);
                            if list_depth == 0 {
                                content_styled.extend(Text::from(Line::from(""))); // Add blank line after list
                            }
                        }

                        TagEnd::Item => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                        }

                        TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link { .. } => {
                            style_stack.pop();
                        }

                        _ => {}
                    }
                }

                Event::SoftBreak => {
                    current_line.push(Span::raw(" "));
                }

                Event::HardBreak => {
                    self.flush_current_line(&mut current_line, &mut content_styled);
                }

                Event::Rule => {
                    self.flush_current_line(&mut current_line, &mut content_styled);
                    let rule = "─".repeat(80);
                    content_styled.extend(Text::from(Line::from(Span::styled(rule, Style::default().fg(Color::Gray)))));
                    content_styled.extend(Text::from(Line::from("")));
                }

                _ => {}
            }
        }

        // Flush any remaining content
        self.flush_current_line(&mut current_line, &mut content_styled);

        self.loaded_blog = Some(content_styled.clone());
        
        // Update scrollbar state if available
        if let Some(ref mut scrollbar_state) = self.scrollbar_state {
            *scrollbar_state = ratatui::widgets::ScrollbarState::new(content_styled.height())
                .position(self.vertical_scroll);
        } else {
            self.scrollbar_state = Some(
                ratatui::widgets::ScrollbarState::new(content_styled.height())
                    .position(self.vertical_scroll)
            );
        }
    }

    fn flush_current_line(&self, current_line: &mut Vec<Span<'static>>, content_styled: &mut Text<'static>) {
        if !current_line.is_empty() {
            content_styled.extend(Text::from(Line::from(current_line.clone())));
            current_line.clear();
        }
    }

    fn render_code_block(&self, code: &str, language: &Option<String>, content_styled: &mut Text) {
        // Add top border
        let border_style = Style::default().fg(Color::Gray);
        content_styled.extend(Text::from(Line::from(Span::styled("┌─────────────────", border_style))));

        if let Some(lang) = language {
            if let Ok(syntax) = self.syntax_set.find_syntax_by_extension(lang)
                .or_else(|| self.syntax_set.find_syntax_by_name(lang))
                .ok_or("Syntax not found") 
            {
                let theme = &self.theme_set.themes["base16-ocean.dark"];
                let mut highlighter = HighlightLines::new(syntax, theme);

                for line in LinesWithEndings::from(code) {
                    let mut line_spans = vec![Span::styled("│ ", border_style)];
                    
                    if let Ok(highlights) = highlighter.highlight_line(line, &self.syntax_set) {
                        for (style, text) in highlights {
                            let ratatui_style = self.syntect_to_ratatui_style(style);
                            line_spans.push(Span::styled(text.to_string(), ratatui_style));
                        }
                    } else {
                        // Fallback to plain text if highlighting fails
                        line_spans.push(Span::raw(line.to_string()));
                    }

                    content_styled.extend(Text::from(Line::from(line_spans)));
                }
            } else {
                // Fallback for unknown languages
                self.render_plain_code_block(code, content_styled, border_style);
            }
        } else {
            // No language specified, render as plain text
            self.render_plain_code_block(code, content_styled, border_style);
        }

        // Add bottom border
        content_styled.extend(Text::from(Line::from(Span::styled("└─────────────────", border_style))));
    }

    fn render_plain_code_block(&self, code: &str, content_styled: &mut Text, border_style: Style) {
        let code_style = Style::default()
            .bg(Color::Rgb(40, 40, 40))
            .fg(Color::Rgb(220, 220, 220));

        for line in code.lines() {
            let mut line_spans = vec![Span::styled("│ ", border_style)];
            line_spans.push(Span::styled(line.to_string(), code_style));
            content_styled.extend(Text::from(Line::from(line_spans)));
        }
    }

    fn syntect_to_ratatui_style(&self, syntect_style: SyntectStyle) -> Style {
        let fg_color = Color::Rgb(
            syntect_style.foreground.r,
            syntect_style.foreground.g,
            syntect_style.foreground.b,
        );

        let mut style = Style::default().fg(fg_color);

        if syntect_style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
            style = style.add_modifier(Modifier::BOLD);
        }
        if syntect_style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if syntect_style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        style
    }

    pub fn scroll_up(&mut self, lines: u16) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(lines as usize);
        if let Some(ref mut scrollbar_state) = self.scrollbar_state {
            *scrollbar_state = scrollbar_state.position(self.vertical_scroll);
        }
    }

    pub fn scroll_down(&mut self, lines: u16) {
        if let Some(ref content) = self.loaded_blog {
            let max_scroll = content.height().saturating_sub(1);
            self.vertical_scroll = (self.vertical_scroll + lines as usize).min(max_scroll as usize);
            if let Some(ref mut scrollbar_state) = self.scrollbar_state {
                *scrollbar_state = scrollbar_state.position(self.vertical_scroll);
            }
        }
    }

    pub fn get_content(&self) -> Option<&Text> {
        self.loaded_blog.as_ref()
    }
}

use pulldown_cmark::{Event, Parser, Tag, TagEnd, CodeBlockKind, TextMergeStream};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

pub struct MarkdownRenderer {
    pub loaded_blog: Option<Text<'static>>,
    pub scrollbar_state: Option<ratatui::widgets::ScrollbarState>,
    pub vertical_scroll: usize,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self {
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
                        for word in text.split_whitespace() {
                            let line_len = current_line.iter().map(|s| s.content.len()).sum::<usize>();
                            if line_len > 0 && line_len + word.len() + 1 > 80 {
                                content_styled.lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                            if !current_line.is_empty() { current_line.push(Span::styled(" ", current_style)); }
                            current_line.push(Span::styled(word.to_string(), current_style));
                        }
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
                            content_styled.extend(Text::from(Line::from("")));
                        }

                        TagEnd::Heading { .. } => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            content_styled.extend(Text::from(Line::from("")));
                            style_stack.pop();
                        }

                        TagEnd::CodeBlock => {
                            if in_code_block {
                                self.render_highlighted_code_block(&code_block_content, &mut content_styled);
                                in_code_block = false;
                                content_styled.extend(Text::from(Line::from("")));
                            }
                        }

                        TagEnd::BlockQuote(_) => {
                            self.flush_current_line(&mut current_line, &mut content_styled);
                            in_blockquote = false;
                            style_stack.pop();
                            content_styled.extend(Text::from(Line::from("")));
                        }

                        TagEnd::List(_) => {
                            list_depth = list_depth.saturating_sub(1);
                            if list_depth == 0 {
                                content_styled.extend(Text::from(Line::from("")));
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

    fn render_highlighted_code_block(&self, code: &str, content_styled: &mut Text) {
        // Add top border
        let border_style = Style::default().fg(Color::Gray);
        content_styled.extend(Text::from(Line::from(Span::styled("┌─────────────────", border_style))));

        // Parse the highlighted code with inline tokens
        for line in code.lines() {
            let mut line_spans = vec![Span::styled("│ ", border_style)];
            
            let mut chars = line.chars().peekable();
            let mut current_text = String::new();
            
            while let Some(ch) = chars.next() {
                if ch == '§' {
                    // Start of a token - first add any accumulated text
                    if !current_text.is_empty() {
                        line_spans.push(Span::styled(
                            current_text.clone(),
                            Style::default().fg(Color::Rgb(220, 220, 220))
                        ));
                        current_text.clear();
                    }
                    
                    // Parse the style code
                    if let Some(style_code) = chars.next() {
                        // Skip the second §
                        if chars.next() == Some('§') {
                            // Read until §/§
                            let mut token_text = String::new();
                            let mut found_end = false;
                            
                            while let Some(ch) = chars.next() {
                                if ch == '§' {
                                    // Check if this is the end marker
                                    if chars.peek() == Some(&'/') {
                                        chars.next(); // consume '/'
                                        if chars.next() == Some('§') {
                                            found_end = true;
                                            break;
                                        }
                                    } else {
                                        token_text.push(ch);
                                    }
                                } else {
                                    token_text.push(ch);
                                }
                            }
                            
                            if found_end {
                                let style = self.code_to_style(style_code);
                                line_spans.push(Span::styled(token_text, style));
                            } else {
                                // Malformed token, just add as plain text
                                current_text.push('§');
                                current_text.push(style_code);
                                current_text.push('§');
                                current_text.push_str(&token_text);
                            }
                        } else {
                            // Malformed token
                            current_text.push('§');
                            current_text.push(style_code);
                        }
                    } else {
                        // Just a lone §
                        current_text.push('§');
                    }
                } else {
                    current_text.push(ch);
                }
            }
            
            // Add any remaining text
            if !current_text.is_empty() {
                line_spans.push(Span::styled(
                    current_text,
                    Style::default().fg(Color::Rgb(220, 220, 220))
                ));
            }
            
            content_styled.extend(Text::from(Line::from(line_spans)));
        }

        // Add bottom border
        content_styled.extend(Text::from(Line::from(Span::styled("└─────────────────", border_style))));
    }

    fn code_to_style(&self, code: char) -> Style {
        match code {
            'k' => Style::default()
                .fg(Color::Rgb(183, 101, 235))  // Purple for keywords
                .add_modifier(Modifier::BOLD),
            's' => Style::default()
                .fg(Color::Rgb(163, 190, 140)),  // Green for strings
            'c' => Style::default()
                .fg(Color::Rgb(143, 161, 179))  // Gray for comments
                .add_modifier(Modifier::ITALIC),
            'n' => Style::default()
                .fg(Color::Rgb(208, 135, 112)),  // Orange for numbers
            'f' => Style::default()
                .fg(Color::Rgb(235, 203, 139))  // Yellow for functions
                .add_modifier(Modifier::BOLD),
            't' => Style::default()
                .fg(Color::Rgb(143, 188, 187)),  // Cyan for types/classes
            'o' => Style::default()
                .fg(Color::Rgb(216, 222, 233)),  // Light gray for operators
            _ => Style::default()
                .fg(Color::Rgb(220, 220, 220)),  // Default white
        }
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

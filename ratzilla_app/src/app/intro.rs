use ratatui::{
    prelude::*,
    widgets::Paragraph,
    layout::Rect,
    Frame,
};
use web_time::{Duration, Instant};

static INTRO_ASCII: &'static str = r"                                 _____          
        ______                  /\    \         
       |::|   |                /::\    \        
       |::|   |               /::::\    \       
       |::|   |              /::::::\    \      
       |::|   |             /:::/\:::\    \     
       |::|   |            /:::/  \:::\    \    
       |::|   |           /:::/    \:::\    \   
       |::|   |          /:::/    / \:::\    \  
 ______|::|___|___ ____ /:::/    /   \:::\ ___\ 
|:::::::::::::::::|    /:::/____/     \:::|    |
|:::::::::::::::::|____\:::\    \     /:::|____|
 ~~~~~~|::|~~~|~~~      \:::\    \   /:::/    / 
       |::|   |          \:::\    \ /:::/    /  
       |::|   |           \:::\    /:::/    /   
       |::|   |            \:::\  /:::/    /    
       |::|   |             \:::\/:::/    /     
       |::|   |              \::::::/    /      
       |::|   |               \::::/    /       
       |::|___|                \::/____/        
        ~~                      ~~              
";
pub struct IntroModel {
    last_frame: Instant,
    total_elapsed: Duration,
    spin_effect: patterns::SpinningGeometry,
    pulse_effect: patterns::GeometricPulse,
}

impl IntroModel {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            total_elapsed: Duration::from_millis(0),
            spin_effect: patterns::SpinningGeometry::new(8000), // 4 second rotation
            pulse_effect: patterns::GeometricPulse::new(4000),  // 2 second pulse
        }
    }

    pub fn update(&mut self, _msg: crate::app::Msg) {
        // Update logic if needed
    }

    pub fn view(&mut self, f: &mut Frame, area: Rect) {
        let elapsed = self.last_frame.elapsed();
        self.last_frame = Instant::now();
        self.total_elapsed += elapsed;

        // // Render the ASCII art once
        // let p = Paragraph::new(INTRO_ASCII)
        //     .alignment(Alignment::Center);
        // f.render_widget(p, area);
        //
        // Update and apply effects in order
        //self.pulse_effect.update(elapsed);
        //self.pulse_effect.apply(f.buffer_mut(), area);
        
        self.spin_effect.update(elapsed);
        self.spin_effect.apply(f.buffer_mut(), area);
    }
}

pub mod patterns {
    use super::*;
    use std::f32::consts::PI;

    /// Creates a spinning geometric pattern with clean lines
    pub struct SpinningGeometry {
        duration_ms: u64,
        elapsed_ms: u64,
    }

    impl SpinningGeometry {
        pub fn new(duration_ms: u64) -> Self {
            Self {
                duration_ms,
                elapsed_ms: 0,
            }
        }

        pub fn update(&mut self, elapsed: Duration) {
            self.elapsed_ms = (self.elapsed_ms + elapsed.as_millis() as u64) % self.duration_ms;
        }

        pub fn apply(&self, buffer: &mut ratatui::buffer::Buffer, area: Rect) {
            let progress = self.elapsed_ms as f32 / self.duration_ms as f32;
            let angle = progress * 2.0 * PI;
            
            let center_x = area.left() + area.width / 2;
            let center_y = area.top() + area.height / 2;
            
            // Create spinning rays
            let num_rays = 4;
            for i in 0..num_rays {
                let ray_angle = angle + (i as f32 * 2.0 * PI / num_rays as f32);
                self.draw_ray(buffer, center_x, center_y, ray_angle, area);
            }
            
            // Add rotating square frames
            self.draw_rotating_square(buffer, center_x, center_y, angle, area);
        }
        
        fn draw_ray(&self, buffer: &mut ratatui::buffer::Buffer, cx: u16, cy: u16, angle: f32, area: Rect) {
            let max_radius = ((area.width.pow(2) + area.height.pow(2)) as f32).sqrt() / 2.0;
            let steps = max_radius as usize;
            
            for step in 0..steps {
                let r = step as f32;
                let x = cx as f32 + r * angle.cos();
                let y = cy as f32 + r * angle.sin() * 0.5; // Adjust for terminal aspect ratio
                
                if x >= area.left() as f32 && x < area.right() as f32 
                   && y >= area.top() as f32 && y < area.bottom() as f32 {
                    if let Some(cell) = buffer.cell_mut((x as u16, y as u16)) {
                        // Use different characters based on distance for gradient effect
                        let char = match (r / max_radius * 4.0) as usize {
                            0 => '●',
                            1 => '◉',
                            2 => '○',
                            _ => '·',
                        };
                        cell.set_char(char);
                    }
                }
            }
        }
        
        fn draw_rotating_square(&self, buffer: &mut ratatui::buffer::Buffer, cx: u16, cy: u16, angle: f32, area: Rect) {
            // Draw multiple concentric rotating squares
            let sizes = [5.0, 10.0, 15.0, 20.0];
            
            for (i, &size) in sizes.iter().enumerate() {
                let rotation = angle + (i as f32 * PI / 8.0); // Offset rotation for each square
                
                // Calculate the four corners of the square
                let corners = [
                    (size, size),
                    (size, -size),
                    (-size, -size),
                    (-size, size),
                ];
                
                // Rotate and draw lines between corners
                for j in 0..4 {
                    let (x1, y1) = self.rotate_point(corners[j].0, corners[j].1, rotation);
                    let (x2, y2) = self.rotate_point(corners[(j + 1) % 4].0, corners[(j + 1) % 4].1, rotation);
                    
                    self.draw_line(
                        buffer,
                        cx as f32 + x1,
                        cy as f32 + y1 * 0.5,
                        cx as f32 + x2,
                        cy as f32 + y2 * 0.5,
                        area,
                        '◈'
                    );
                }
            }
        }
        
        fn rotate_point(&self, x: f32, y: f32, angle: f32) -> (f32, f32) {
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            (x * cos_a - y * sin_a, x * sin_a + y * cos_a)
        }
        
        fn draw_line(&self, buffer: &mut ratatui::buffer::Buffer, x1: f32, y1: f32, x2: f32, y2: f32, area: Rect, char: char) {
            let steps = ((x2 - x1).abs().max((y2 - y1).abs()) * 2.0) as usize + 1;
            
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let x = x1 + (x2 - x1) * t;
                let y = y1 + (y2 - y1) * t;
                
                if x >= area.left() as f32 && x < area.right() as f32 
                   && y >= area.top() as f32 && y < area.bottom() as f32 {
                    if let Some(cell) = buffer.cell_mut((x as u16, y as u16)) {
                        cell.set_char(char);
                    }
                }
            }
        }
    }

    /// Geometric pulse effect that expands and contracts in a structured way
    pub struct GeometricPulse {
        duration_ms: u64,
        elapsed_ms: u64,
    }

    impl GeometricPulse {
        pub fn new(duration_ms: u64) -> Self {
            Self {
                duration_ms,
                elapsed_ms: 0,
            }
        }

        pub fn update(&mut self, elapsed: Duration) {
            self.elapsed_ms = (self.elapsed_ms + elapsed.as_millis() as u64) % self.duration_ms;
        }

        pub fn apply(&self, buffer: &mut ratatui::buffer::Buffer, area: Rect) {
            let progress = self.elapsed_ms as f32 / self.duration_ms as f32;
            let pulse = (progress * 2.0 * PI).sin().abs();
            
            let center_x = area.left() + area.width / 2;
            let center_y = area.top() + area.height / 2;
            
            // Create hexagon pulse
            self.draw_hexagon(buffer, center_x, center_y, pulse, area);
            
            // Add corner accents that pulse
            self.draw_corner_accents(buffer, area, pulse);
        }
        
        fn draw_hexagon(&self, buffer: &mut ratatui::buffer::Buffer, cx: u16, cy: u16, pulse: f32, area: Rect) {
            let base_radius = 8.0 + pulse * 12.0;
            let vertices = 6;
            
            for i in 0..vertices {
                let angle1 = (i as f32 * 2.0 * PI / vertices as f32) - PI / 2.0;
                let angle2 = ((i + 1) as f32 * 2.0 * PI / vertices as f32) - PI / 2.0;
                
                let x1 = cx as f32 + base_radius * angle1.cos();
                let y1 = cy as f32 + base_radius * angle1.sin() * 0.5;
                let x2 = cx as f32 + base_radius * angle2.cos();
                let y2 = cy as f32 + base_radius * angle2.sin() * 0.5;
                
                self.draw_line(buffer, x1, y1, x2, y2, area, '⬡');
            }
        }
        
        fn draw_corner_accents(&self, buffer: &mut ratatui::buffer::Buffer, area: Rect, pulse: f32) {
            let size = (3.0 + pulse * 2.0) as u16;
            let chars = ['┌', '┐', '└', '┘'];
            let positions = [
                (area.left() + size, area.top() + size),
                (area.right() - size - 1, area.top() + size),
                (area.left() + size, area.bottom() - size - 1),
                (area.right() - size - 1, area.bottom() - size - 1),
            ];
            
            for (i, &(x, y)) in positions.iter().enumerate() {
                if x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom() {
                    if let Some(cell) = buffer.cell_mut((x, y)) {
                        cell.set_char(chars[i]);
                    }
                }
            }
        }
        
        fn draw_line(&self, buffer: &mut ratatui::buffer::Buffer, x1: f32, y1: f32, x2: f32, y2: f32, area: Rect, char: char) {
            let steps = ((x2 - x1).abs().max((y2 - y1).abs()) * 2.0) as usize + 1;
            
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let x = x1 + (x2 - x1) * t;
                let y = y1 + (y2 - y1) * t;
                
                if x >= area.left() as f32 && x < area.right() as f32 
                   && y >= area.top() as f32 && y < area.bottom() as f32 {
                    if let Some(cell) = buffer.cell_mut((x as u16, y as u16)) {
                        cell.set_char(char);
                    }
                }
            }
        }
    }
}

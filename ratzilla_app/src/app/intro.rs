use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    layout::Rect,
    Frame,
};
use tachyonfx::{fx, Effect, CellFilter, EffectManager, Interpolation};
use web_time::Instant;


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
pub struct IntroModel{
    last_frame: Instant,
    fx_manager: EffectManager<()>,
}

impl IntroModel {
    pub fn new() -> Self {
        let c = Color::from_u32(0x1d2021);
        let timer = (1000, Interpolation::Linear);
        let fg_shift = [120.0, 25.0, 25.0];
        let bg_shift = [-40.0, -50.0, -50.0];
        let fx_seq = fx::sequence(&[
            fx::coalesce(2222),
        ]);
        let fx_para = fx::parallel(&[
            fx_seq.clone(),
            //fx::slide_in(tachyonfx::Motion::DownToUp, 10, 909, Color::Black, 2000),
        ]);

        let mut manager: EffectManager<()> = EffectManager::default();
        manager.add_effect(fx_para);
        Self {
            last_frame: Instant::now(),
            fx_manager: manager,
        }
    }

    pub fn update(&mut self, _msg: crate::app::Msg) {
    }

    pub fn view(&mut self, f: &mut Frame, area: Rect) {
        let elapsed = self.last_frame.elapsed();
        self.last_frame = Instant::now();

        let p = Paragraph::new(INTRO_ASCII)
            .alignment(Alignment::Center);

        f.render_widget(p, area);

        self.fx_manager.process_effects(elapsed.into(), f.buffer_mut(), area);
    }
}

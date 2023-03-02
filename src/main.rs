use std::{time::Duration, sync::{Mutex, Arc}, io};

use rodio::{OutputStream, Source};



use rand::{thread_rng, Rng};

use std::io::Read;
use std::time::Instant;
use termion::{async_stdin, event::Key, input::TermRead, raw::IntoRawMode};
use tui::{backend::TermionBackend, symbols, widgets::LineGauge, style::Modifier};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;

fn main()  -> Result<(), io::Error> {

    // Set up terminal output
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut asi = async_stdin();

    let generator = BinauralGenerator::new(120,40,44100);



    // let mut generator: BinauralGenerator = BinauralGenerator::new(120, 40, 44100);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let gen_next_vol = Arc::clone(&generator.next_vol);
    let _result = stream_handle.play_raw(generator.convert_samples());
    std::thread::sleep(std::time::Duration::from_secs(1));

    terminal.clear()?;
    loop {

        // Lock the terminal and start a drawing session.
        terminal.draw(|frame| {
            let mut volume_percent:u16 = 0;
            if let Ok(inner_vol) = gen_next_vol.lock(){
                volume_percent = (*inner_vol).vol_percent();
            }
            // Create a layout into which to place our blocks.
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(100),
                    ]
                    .as_ref(),
                )
                .split(frame.size());
            let gauge = LineGauge::default()
            .block(Block::default().borders(Borders::ALL).title("Volume"))
            .gauge_style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD))
            .line_set(symbols::line::THICK)
            .ratio((volume_percent as f64)/100.);
            frame.render_widget(gauge, chunks[0]);

        })?;

        // Iterate over all the keys that have been pressed since the
        // last time we checked.
        for k in asi.by_ref().keys() {
            match k.unwrap() {
                // If any of them is q, quit
                Key::Char('q') => {
                    // Clear the terminal before exit so as not to leave
                    // a mess.
                    terminal.clear()?;
                    return Ok(());
                }
                Key::Right => {
                    if let Ok(mut inner_vol) = gen_next_vol.lock(){
                        _ = (*inner_vol).vol_up();
                    }
                }
                Key::Left => {
                    if let Ok(mut inner_vol) = gen_next_vol.lock(){
                        _ = (*inner_vol).vol_down();
                    }
                }
                // Otherwise, throw them away.
                _ => (),
            }
        }
    }

    Ok(())
}


pub trait Vol{
    const MAX:f32 = 1.;
    const MIN:f32 = 0.;


    fn vol_up(&mut self)-> Result<(),()>;
    fn vol_down(&mut self)-> Result<(),()>;
    fn volume(&self)->f32;
    fn vol_percent(&self) -> u16;
}

struct Volume{
    vol: f32,
}
impl Volume{
    fn new() -> Self{
        Volume { vol: 0.5 }
    }
    
}

impl Vol for Volume{
    fn vol_up(&mut self)-> Result<(),()> {
        if self.vol < Volume::MAX{
            self.vol += 0.01;
            if self.vol > Volume::MAX{
                self.vol = Volume::MAX;
            }
            return Ok(());
        }
        Err(())
    }

    fn vol_down(&mut self)-> Result<(),()> {
        if self.vol > Volume::MIN{
            self.vol -= 0.01;
            if self.vol < Volume::MIN{
                self.vol = Volume::MIN;
            }
            return Ok(());
        }
        Err(())
    }

    fn volume(&self)->f32 {
        self.vol
    }

    fn vol_percent(&self) -> u16{
        (100. * self.vol/Volume::MAX).round() as u16
    }
}

struct BinauralGenerator{
    center_freq: i32,
    binaural_freq: i32, 
    sample_rate: i32,
    f_high: i32,
    f_low: i32,
    cur_index: u32,
    cur_index_low: u32,
    cur_high:bool,
    vol: f32,
    pub next_vol: Arc<Mutex<Volume>>,
}

impl BinauralGenerator{
    fn new(center_freq: i32, binaural_freq: i32, sample_rate: i32 ) -> Self{
        let f_high = center_freq + (binaural_freq/2);
        let f_low = center_freq - (binaural_freq/2);
    
        // Calculate a window that encompasses whole wavelengths of each frequency for looping.
        // Least common multiple would be optimal, but just multiplying the two numbers is easier.
        BinauralGenerator { 
            center_freq,
            binaural_freq, 
            sample_rate, 
            f_high,
            f_low,
            cur_index : 0,
            cur_index_low: 0,
            cur_high: false,
            vol: 1.,
            next_vol: Arc::new(Mutex::new(Volume::new())),
        }
    }
}

impl Source for BinauralGenerator{
    fn current_frame_len(&self) -> Option<usize>{
        None
    }
    fn channels(&self) -> u16{
        2
    }
    fn sample_rate(&self) -> u32{
        self.sample_rate as u32
    }
    fn total_duration(&self) -> Option<Duration>{
        None
    }
}

impl Iterator for BinauralGenerator{
    type Item = f32;
     
    fn next(&mut self) -> Option<f32>{
        if let Ok(n_vol) = self.next_vol.try_lock(){
            self.vol = (*n_vol).volume().clone();
        }
        let sample: Option<f32>;
        if !self.cur_high{
            sample = Some ( (((self.cur_index_low as f32)/(self.sample_rate as f32))* 2. * std::f32::consts::PI * (self.f_low as f32)).sin()*self.vol);
            self.cur_index_low = self.cur_index_low.wrapping_add(1);
        }else{
            sample = Some ( (((self.cur_index as f32)/(self.sample_rate as f32))* 2. * std::f32::consts::PI * (self.f_high as f32)).sin()*self.vol);
            self.cur_index = self.cur_index.wrapping_add(1);
        }
        self.cur_high = !self.cur_high;
        sample
    }
}
use std::{sync:: Arc, io::{self, Read}};


use rodio::{OutputStream, Source};



use termion::{async_stdin, event::Key, input::TermRead, raw::IntoRawMode};
use tui::{backend::TermionBackend, symbols, widgets::LineGauge, style::Modifier};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders};
use tui::Terminal;

use oxibeats::audio::{BinauralGenerator, Vol};

fn main()  -> Result<(), io::Error> {


    let generator = BinauralGenerator::new(120,40,44100);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let gen_next_vol = Arc::clone(&generator.next_vol);
    let _result = stream_handle.play_raw(generator.convert_samples());

    // Set up terminal output
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut asi = async_stdin();

 
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
                        Constraint::Length(3),
                        Constraint::Max(4),
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

}

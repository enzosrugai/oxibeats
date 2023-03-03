
pub mod audio{
    use std::{sync::{Mutex, Arc}, time::Duration};

    use rodio::Source;


    pub trait Vol{
        const MAX:f32 = 1.;
        const MIN:f32 = 0.;


        fn vol_up(&mut self)-> Option<u16>;
        fn vol_down(&mut self)-> Option<u16>;
        fn volume(&self)->f32;
        fn vol_percent(&self) -> u16;
    }


    pub struct Volume{
        vol: f32,
    }
    impl Volume{
        fn new() -> Self{
            Volume { vol: 0.5 }
        }
        
    }


    impl Vol for Volume{
        fn vol_up(&mut self)-> Option<u16>{
            if self.vol < Volume::MAX{
                self.vol += 0.01;
                if self.vol > Volume::MAX{
                    self.vol = Volume::MAX;
                }
                return Some(self.vol_percent());
            }
            None
        }

        fn vol_down(&mut self)-> Option<u16>{
            if self.vol > Volume::MIN{
                self.vol -= 0.01;
                if self.vol < Volume::MIN{
                    self.vol = Volume::MIN;
                }
                return Some(self.vol_percent())
            }
            None
        }

        fn volume(&self)->f32 {
            self.vol
        }

        fn vol_percent(&self) -> u16{
            (100. * self.vol/Volume::MAX).round() as u16
        }
    }

    pub struct BinauralGenerator{
        center_freq: u32,
        binaural_freq: u32, 
        sample_rate: u32,
        sine_high: SineWaveform,
        sine_low: SineWaveform,
        cur_high:bool,
        vol: f32,
        pub next_vol: Arc<Mutex<Volume>>,
    }

    struct SineWaveform{
        freq : u32,
        wavetable : Vec<f32>,
        table_length : u32,
        cur_index : u32,
    }
    impl SineWaveform{
        pub fn new(freq:u32, sample_rate:u32) -> Self{
            let table_length = sample_rate/freq;
            let wavetable = (0..table_length).map(|n| (((n as f32)/(sample_rate as f32))* 2. * std::f32::consts::PI * (freq as f32)).sin()).collect();
            let cur_index = 0;
            SineWaveform { freq, wavetable, table_length, cur_index}
        }
    }
    impl Iterator for SineWaveform{
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            let sample = *self.wavetable.get(self.cur_index as usize)?;
            self.cur_index += 1;
            if self.cur_index >= self.table_length{
                self.cur_index = 0;
            }
            Some(sample)
        }
    }
    impl BinauralGenerator{
        pub fn new(center_freq: u32, binaural_freq: u32, sample_rate: u32 ) -> Self{
            let f_high = center_freq + (binaural_freq/2);
            let f_low = center_freq - (binaural_freq/2);
        
            // Calculate a window that encompasses whole wavelengths of each frequency for looping.
            // Least common multiple would be optimal, but just multiplying the two numbers is easier.
            BinauralGenerator { 
                center_freq,
                binaural_freq, 
                sample_rate, 
                sine_high: SineWaveform::new(f_high, sample_rate),
                sine_low: SineWaveform::new(f_low, sample_rate),
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
            self.sample_rate
        }
        fn total_duration(&self) -> Option<Duration>{
            None
        }
    }

    impl Iterator for BinauralGenerator{
        type Item = f32;
        
        fn next(&mut self) -> Option<f32>{
            if let Ok(n_vol) = self.next_vol.try_lock(){
                self.vol = (*n_vol).volume();
            }

            let sample: f32 =    if self.cur_high{
                self.sine_high.next()? * self.vol
            }else{
                self.sine_low.next()? * self.vol
            };

            self.cur_high = !self.cur_high;
            Some(sample)
        }
    }
}
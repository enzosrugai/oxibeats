
pub mod audio{
    use std::{sync::{Mutex, Arc}, time::Duration};

    use rodio::Source;


    pub trait Vol{
        const MAX:f32 = 1.;
        const MIN:f32 = 0.;


        fn vol_up(&mut self)-> Result<(),()>;
        fn vol_down(&mut self)-> Result<(),()>;
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

    pub struct BinauralGenerator{
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
        pub fn new(center_freq: i32, binaural_freq: i32, sample_rate: i32 ) -> Self{
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
}
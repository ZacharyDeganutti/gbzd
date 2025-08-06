use sdl3;
use std::{collections::VecDeque, sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex}};

#[derive(Copy, Clone, Debug)]
pub enum DutyCycle {
    Eighth,
    Quarter,
    Half,
    ThreeQuarter
}

#[derive(Copy, Clone, Debug)]
pub struct SquareWave {
    pub duty_cycle: DutyCycle,
    // 0 to 1 range
    pub volume: f32,
    // in hz
    pub frequency: f32,
}

// A wave made of floating point samples
#[derive(Copy, Clone, Debug)]
pub struct SampleWave<const COUNT: usize> {
    // 0 to 1 range
    pub volume_samples: [f32; COUNT],
    // in hz
    pub frequency: f32,
}

// A wave made of floating point samples
#[derive(Clone, Debug)]
pub struct NoiseWave {
    // 0 to 1 range
    pub volume_samples: Arc<Mutex<VecDeque<f32>>>,
}

// SDL-specific implementation details below
pub struct SquareGenerator {
    wave_inbox: Receiver<SquareWave>,
    wave: SquareWave,
    phase: f32,
    sample_rate: f32
}

impl sdl3::audio::AudioCallback<f32> for SquareGenerator {
    
    fn callback(&mut self, out: &mut [f32]) {
        // Check wave data for most recently posted value, otherwise use the cached value
        let mut wave = self.wave;
        self.wave = loop { 
            // chew through the queue until there's nothing left and break on the last good value
            match self.wave_inbox.try_recv() {
                Ok(new_wave) => wave = new_wave,
                Err(_) => break wave
            }
        };

        let hi_cutoff = match self.wave.duty_cycle {
            DutyCycle::Eighth => 0.825,
            DutyCycle::Quarter => 0.75,
            DutyCycle::Half => 0.5,
            DutyCycle::ThreeQuarter => 0.175
        };

        for sample in out.iter_mut() {
            *sample = if self.phase <= hi_cutoff {
                self.wave.volume
            } 
            else {
                -self.wave.volume
            };
            let phase_inc = self.wave.frequency / self.sample_rate;
            self.phase = (self.phase + phase_inc) % 1.0;
        }
    }
}


pub struct SampleGenerator<const COUNT: usize> {
    wave_inbox: Receiver<SampleWave<COUNT>>,
    wave: SampleWave<COUNT>,
    phase: f32,
    sample_rate: f32
}

impl<const COUNT: usize> sdl3::audio::AudioCallback<f32> for SampleGenerator<COUNT> {
    
    fn callback(&mut self, out: &mut [f32]) {
        // Check wave data for most recently posted value, otherwise use the cached value
        let mut wave = self.wave;
        self.wave = loop { 
            // chew through the queue until there's nothing left and break on the last good value
            match self.wave_inbox.try_recv() {
                Ok(new_wave) => wave = new_wave,
                Err(_) => break wave
            }
        };

        for sample in out.iter_mut() {
            let sample_index = (COUNT as f32 * self.phase) as usize;  
            *sample = self.wave.volume_samples[sample_index];
            let phase_inc = self.wave.frequency / self.sample_rate;
            self.phase = (self.phase + phase_inc) % 1.0;
        }
    }
}

pub struct NoiseGenerator {
    wave_inbox: Receiver<NoiseWave>,
}

impl sdl3::audio::AudioCallback<f32> for NoiseGenerator {
    
    fn callback(&mut self, out: &mut [f32]) {
        // Check wave data for most recently posted value, otherwise use the cached value
        let mut wave: Option<NoiseWave> = None;
        loop { 
            // chew through the queue until there's nothing left and break on the last good value
            match self.wave_inbox.try_recv() {
                Ok(new_wave) => wave = Some(new_wave),
                Err(_) => break
            };
        };


        match wave {
            Some(noise_data) => {
                let noise_unwrapped = noise_data.volume_samples.lock().unwrap();
                let noise_slice = noise_unwrapped.as_slices().0;
                let noise_slice_2 = noise_unwrapped.as_slices().1;

                // Through the power of being lazy, these should both be length 441
                for idx in 0..noise_slice.len() {
                    out[idx] = noise_slice[idx];
                }
                for idx in 0..noise_slice_2.len() {
                    out[idx + noise_slice.len()] = noise_slice_2[idx];
                }
            }
            None => {
                for idx in 0..out.iter_mut().len() {
                    out[idx] = 0.0;
                }
            }
        }
        
    }
}


pub struct GbAudioSdl {
    audio_subsystem: sdl3::AudioSubsystem,
    spec: sdl3::audio::AudioSpec,
    playback_device: sdl3::audio::AudioDevice,
    channel_1_outbox: Sender<SquareWave>,
    channel_1: sdl3::audio::AudioStreamWithCallback<SquareGenerator>,
    channel_2_outbox: Sender<SquareWave>,
    channel_2: sdl3::audio::AudioStreamWithCallback<SquareGenerator>,
    channel_3_outbox: Sender<SampleWave<32>>,
    channel_3: sdl3::audio::AudioStreamWithCallback<SampleGenerator<32>>,
    channel_4_outbox: Sender<NoiseWave>,
    channel_4: sdl3::audio::AudioStreamWithCallback<NoiseGenerator>
}

impl GbAudioSdl {
    pub fn new(sdl_context: &sdl3::Sdl) -> GbAudioSdl {
        let audio_subsystem = sdl_context.audio().unwrap();
        let playback_device = audio_subsystem.default_playback_device();
        let spec = sdl3::audio::AudioSpec {
            freq: Some(44100),
            channels: Some(1),
            format: Some(sdl3::audio::AudioFormat::f32_sys())
        };
        let default_square_wave = SquareWave {
            duty_cycle: DutyCycle::Quarter,
            volume: 0.03,
            frequency: 440.0,
        };
        let default_sample_wave = SampleWave {
            volume_samples: [0.0; 32],
            frequency: 440.0,
        };

        let (wave_1_outbox, wave_1_inbox) = channel();
        let channel_1_wave_generator = SquareGenerator {
            wave: default_square_wave,
            wave_inbox: wave_1_inbox,
            phase: 0.0,
            sample_rate: 44100.0,
        };
        let channel_1_stream = audio_subsystem.open_playback_stream_with_callback(&playback_device, &spec, channel_1_wave_generator).unwrap();

        let (wave_2_outbox, wave_2_inbox) = channel();
        let channel_2_wave_generator = SquareGenerator {
            wave: default_square_wave,
            wave_inbox: wave_2_inbox,
            phase: 0.0,
            sample_rate: 44100.0,
        };
        let channel_2_stream = audio_subsystem.open_playback_stream_with_callback(&playback_device, &spec, channel_2_wave_generator).unwrap();

        let (wave_3_outbox, wave_3_inbox) = channel();
        let channel_3_wave_generator = SampleGenerator {
            wave: default_sample_wave,
            wave_inbox: wave_3_inbox,
            phase: 0.0,
            sample_rate: 44100.0,
        };
        let channel_3_stream = audio_subsystem.open_playback_stream_with_callback(&playback_device, &spec, channel_3_wave_generator).unwrap();

        let (wave_4_outbox, wave_4_inbox) = channel();
        let channel_4_wave_generator = NoiseGenerator {
            wave_inbox: wave_4_inbox,
        };
        let channel_4_stream = audio_subsystem.open_playback_stream_with_callback(&playback_device, &spec, channel_4_wave_generator).unwrap();

        GbAudioSdl { 
            audio_subsystem,
            spec,
            playback_device,
            channel_1_outbox: wave_1_outbox,
            channel_1: channel_1_stream,
            channel_2_outbox: wave_2_outbox,
            channel_2: channel_2_stream,
            channel_3_outbox: wave_3_outbox,
            channel_3: channel_3_stream,
            channel_4_outbox: wave_4_outbox,
            channel_4: channel_4_stream
        }
    }

    pub fn start_channel_1(&mut self, wave: SquareWave) {
        self.channel_1_outbox.send(wave).unwrap();
        self.channel_1.resume().unwrap();
    }

    pub fn update_channel_1(&mut self, wave: SquareWave) {
        self.channel_1_outbox.send(wave).unwrap();
    }

    pub fn stop_channel_1(&self) {
        self.channel_1.pause().unwrap();
    }

    pub fn start_channel_2(&mut self, wave: SquareWave) {
        self.channel_2_outbox.send(wave).unwrap();
        self.channel_2.resume().unwrap();
    }

    pub fn update_channel_2(&mut self, wave: SquareWave) {
        self.channel_2_outbox.send(wave).unwrap();
    }

    pub fn stop_channel_2(&self) {
        self.channel_2.pause().unwrap();
    }

    pub fn start_channel_3(&mut self, wave: SampleWave<32>) {
        self.channel_3_outbox.send(wave).unwrap();
        self.channel_3.resume().unwrap();
    }

    pub fn update_channel_3(&mut self, wave: SampleWave<32>) {
        self.channel_3_outbox.send(wave).unwrap();
    }

    pub fn stop_channel_3(&self) {
        self.channel_3.pause().unwrap();
    }

    pub fn start_channel_4(&mut self) {
        self.channel_4.resume().unwrap();
    }

    pub fn update_channel_4(&mut self, wave: NoiseWave) {
        self.channel_4_outbox.send(wave).unwrap();
    }

    pub fn stop_channel_4(&self) {
        self.channel_4.pause().unwrap();
    }
}


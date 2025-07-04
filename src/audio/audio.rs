use sdl3;
use std::sync::mpsc::{channel, Receiver, Sender};

pub trait IsWave {
    
}

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

        GbAudioSdl { 
            audio_subsystem,
            spec,
            playback_device,
            channel_1_outbox: wave_1_outbox,
            channel_1: channel_1_stream,
            channel_2_outbox: wave_2_outbox,
            channel_2: channel_2_stream,
            channel_3_outbox: wave_3_outbox,
            channel_3: channel_3_stream
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
}


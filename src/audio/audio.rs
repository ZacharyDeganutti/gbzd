use sdl3;
use std::sync::mpsc::{channel, Receiver, Sender};

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

pub trait ServeAudio {
    fn start_channel_1(&mut self, wave: SquareWave);
    fn update_channel_1(&mut self, wave: SquareWave);
    fn stop_channel_1(&self);
}

pub struct AudioPlayer<T: ServeAudio> {
    audio_service: T
}

impl<T: ServeAudio> AudioPlayer<T> {
    pub fn new(audio_service: T) -> AudioPlayer<T> {
        AudioPlayer { audio_service }
    }

    pub fn start_channel_1(&mut self, wave: SquareWave) {
        self.audio_service.start_channel_1(wave);
    }

    pub fn update_channel_1(&mut self, wave: SquareWave) {
        self.audio_service.update_channel_1(wave);
    }

    pub fn stop_channel_1(&mut self) {
        self.audio_service.stop_channel_1();
    }
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

pub struct SdlAudio {
    audio_subsystem: sdl3::AudioSubsystem,
    spec: sdl3::audio::AudioSpec,
    playback_device: sdl3::audio::AudioDevice,
    channel_1_outbox: Sender<SquareWave>,
    channel_1: sdl3::audio::AudioStreamWithCallback<SquareGenerator>
}

impl ServeAudio for SdlAudio {
    fn start_channel_1(&mut self, wave: SquareWave) {
        self.channel_1_outbox.send(wave).unwrap();
        //self.audio_subsystem.open_playback_stream_with_callback(&self.playback_device, &self.spec, wave).unwrap();
        self.channel_1.resume().unwrap();
    }

    fn update_channel_1(&mut self, wave: SquareWave) {
        self.channel_1_outbox.send(wave).unwrap();
    }

    fn stop_channel_1(&self) {
        self.channel_1.pause().unwrap();
    }
}

impl SdlAudio {
    pub fn new(sdl_context: &sdl3::Sdl) -> SdlAudio {
        let audio_subsystem = sdl_context.audio().unwrap();
        let playback_device = audio_subsystem.default_playback_device();
        let spec = sdl3::audio::AudioSpec {
            freq: Some(44100),
            channels: Some(1),
            format: Some(sdl3::audio::AudioFormat::f32_sys())
        };
        let default_wave = SquareWave {
            duty_cycle: DutyCycle::Quarter,
            volume: 0.03,
            frequency: 440.0,
        };

        let (wave_outbox, wave_inbox) = channel();
        let channel_1_wave_generator = SquareGenerator {
            wave: default_wave,
            wave_inbox,
            phase: 0.0,
            sample_rate: 44100.0,
        };
        let channel_1_stream = audio_subsystem.open_playback_stream_with_callback(&playback_device, &spec, channel_1_wave_generator).unwrap();

        SdlAudio { 
            audio_subsystem,
            spec,
            playback_device,
            channel_1_outbox: wave_outbox,
            channel_1: channel_1_stream
        }
    }
}


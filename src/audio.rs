use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, Stream};
use hound::{WavSpec, WavWriter};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

pub struct AudioRecorder {
    stream: Stream,
    receiver: Receiver<f32>,
}

impl AudioRecorder {
    pub fn start_recording() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow!("Failed to get default input device"))?;

        let user_config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        };

        let (sender, receiver) = mpsc::channel();

        let stream = match device.build_input_stream(
            &user_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for &sample in data {
                    let _ = sender.send(sample);
                }
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None,
        ) {
            Ok(s) => s,
            Err(_) => {
                // Fallback to default config and conversion if 16kHz mono fails natively
                let default_config = device.default_input_config()?;
                let (sender, receiver) = mpsc::channel();
                Self::build_fallback_stream(&device, &default_config, sender)?
            }
        };

        stream.play()?;

        Ok(Self { stream, receiver })
    }

    fn build_fallback_stream(
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        sender: Sender<f32>,
    ) -> Result<Stream> {
        let channels = config.channels();
        let sample_rate = config.sample_rate().0;

        let stream_config = config.config();
        Ok(match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| Self::write_input_data(data, channels, sender.clone()),
                |err| eprintln!("an error occurred on stream: {}", err),
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &_| Self::write_input_data(data, channels, sender.clone()),
                |err| eprintln!("an error occurred on stream: {}", err),
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &stream_config,
                move |data: &[u16], _: &_| Self::write_input_data(data, channels, sender.clone()),
                |err| eprintln!("an error occurred on stream: {}", err),
                None,
            )?,
            sample_format => {
                return Err(anyhow!("Unsupported sample format '{sample_format}'"))
            }
        })
    }

    fn write_input_data<T>(input: &[T], channels: u16, sender: Sender<f32>)
    where
        T: Sample,
    {
        // simplistic downmix to mono by averaging channels if > 1
        for frame in input.chunks(channels as usize) {
            let mut sum = 0.0;
            for sample in frame {
                sum += sample.to_f32();
            }
            let _ = sender.send(sum / channels as f32);
        }
    }

    pub fn stop_recording_and_save(self, save_path: &str) -> Result<()> {
        let _ = self.stream.pause();
        
        // At this point we can drain the channel
        let mut samples: Vec<f32> = Vec::new();
        while let Ok(sample) = self.receiver.try_recv() {
            samples.push(sample);
        }

        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(save_path, spec)?;
        for sample in samples {
            // Convert f32 back to i16 for whisper
            let amplitude = i16::MAX as f32;
            let i16_sample = (sample * amplitude).clamp(-amplitude, amplitude) as i16;
            writer.write_sample(i16_sample)?;
        }
        writer.finalize()?;

        Ok(())
    }
}

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, SampleRate, StreamConfig};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::State;

use crate::AppState;

// Audio device info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

// Recording state - simplified without Stream
#[derive(Default)]
pub struct RecordingState {
    pub sample_rate: u32,
    pub channels: u16,
}

// Global recording control - Stream must stay on the thread that created it
static RECORDING_ACTIVE: AtomicBool = AtomicBool::new(false);
static STOP_SIGNAL: AtomicBool = AtomicBool::new(false);

/// Audio samples result - kept in memory, never written to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSamples {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    println!("Audio host: {:?}", host.id());

    let default_device = host.default_input_device();
    let default_name = default_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();
    println!("Default input device: {}", default_name);

    let mut devices = Vec::new();

    for device in host.input_devices().map_err(|e| e.to_string())? {
        if let Ok(name) = device.name() {
            println!("Found input device: {}", name);
            devices.push(AudioDevice {
                id: name.clone(),
                name: name.clone(),
                is_default: name == default_name,
            });
        }
    }

    // Sort with default first
    devices.sort_by(|a, b| b.is_default.cmp(&a.is_default));

    println!("Total devices found: {}", devices.len());

    Ok(devices)
}

#[tauri::command]
pub fn get_selected_device(state: State<AppState>) -> Result<Option<String>, String> {
    let device = state.selected_audio_device.lock().unwrap().clone();
    Ok(device)
}

#[tauri::command]
pub fn set_selected_device(device_id: String, state: State<AppState>) -> Result<(), String> {
    let mut selected = state.selected_audio_device.lock().unwrap();
    *selected = Some(device_id);
    Ok(())
}

fn get_device_by_name(name: &str) -> Result<Device, String> {
    let host = cpal::default_host();

    for device in host.input_devices().map_err(|e| e.to_string())? {
        if let Ok(device_name) = device.name() {
            if device_name == name {
                return Ok(device);
            }
        }
    }

    Err(format!("Device '{}' not found", name))
}

#[tauri::command]
pub fn start_recording(state: State<AppState>) -> Result<(), String> {
    if RECORDING_ACTIVE.load(Ordering::SeqCst) {
        return Err("Already recording".to_string());
    }

    // Get the selected device name
    let device_name = state.selected_audio_device.lock().unwrap().clone();

    // Get consumer storage
    let consumer_storage = state.sample_consumer.clone();
    let recording_state = state.recording.clone();

    // Reset stop signal
    STOP_SIGNAL.store(false, Ordering::SeqCst);
    RECORDING_ACTIVE.store(true, Ordering::SeqCst);

    // Spawn a thread to handle audio recording
    thread::spawn(move || {
        let result = run_recording(device_name, consumer_storage, recording_state);
        if let Err(e) = result {
            eprintln!("Recording error: {}", e);
        }
        RECORDING_ACTIVE.store(false, Ordering::SeqCst);
    });

    Ok(())
}

fn run_recording(
    device_name: Option<String>,
    consumer_storage: Arc<Mutex<Option<Arc<Mutex<ringbuf::HeapCons<f32>>>>>>,
    recording_state: Arc<Mutex<RecordingState>>,
) -> Result<(), String> {
    // Get the device
    let device = if let Some(ref name) = device_name {
        get_device_by_name(name)?
    } else {
        cpal::default_host()
            .default_input_device()
            .ok_or("No default input device")?
    };

    let device_name = device.name().unwrap_or_default();
    println!("Recording from: {}", device_name);

    // Get supported config
    let supported_config = device
        .supported_input_configs()
        .map_err(|e| e.to_string())?
        .find(|c| c.channels() == 1 && c.min_sample_rate().0 <= 16000 && c.max_sample_rate().0 >= 16000)
        .or_else(|| {
            device
                .supported_input_configs()
                .ok()?
                .find(|c| c.channels() == 1)
        })
        .or_else(|| {
            device
                .supported_input_configs()
                .ok()?
                .next()
        })
        .ok_or("No supported audio config")?;

    // Use 16kHz if supported, otherwise use max sample rate
    let sample_rate = if supported_config.min_sample_rate().0 <= 16000 && supported_config.max_sample_rate().0 >= 16000 {
        SampleRate(16000)
    } else {
        supported_config.max_sample_rate()
    };

    let config = supported_config.with_sample_rate(sample_rate);
    let sample_format = config.sample_format();
    let stream_config: StreamConfig = config.into();

    // Update recording state
    {
        let mut rec = recording_state.lock().unwrap();
        rec.sample_rate = stream_config.sample_rate.0;
        rec.channels = stream_config.channels;
    }

    // Create ring buffer (60 seconds)
    let rb = HeapRb::<f32>::new(stream_config.sample_rate.0 as usize * stream_config.channels as usize * 60);
    let (producer, consumer) = rb.split();

    // Store consumer
    *consumer_storage.lock().unwrap() = Some(Arc::new(Mutex::new(consumer)));

    // Wrap producer for thread safety
    let producer = Arc::new(Mutex::new(producer));

    let err_fn = |err| eprintln!("Audio stream error: {}", err);

    let stream = match sample_format {
        SampleFormat::F32 => {
            let prod = producer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| {
                    if let Ok(mut p) = prod.lock() {
                        for &sample in data {
                            p.try_push(sample).ok();
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            let prod = producer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &_| {
                    if let Ok(mut p) = prod.lock() {
                        for &sample in data {
                            let f = sample as f32 / i16::MAX as f32;
                            p.try_push(f).ok();
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let prod = producer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _: &_| {
                    if let Ok(mut p) = prod.lock() {
                        for &sample in data {
                            let f = (sample as f32 / u16::MAX as f32) * 2.0 - 1.0;
                            p.try_push(f).ok();
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I32 => {
            let prod = producer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i32], _: &_| {
                    if let Ok(mut p) = prod.lock() {
                        for &sample in data {
                            let f = sample as f32 / i32::MAX as f32;
                            p.try_push(f).ok();
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U32 => {
            let prod = producer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[u32], _: &_| {
                    if let Ok(mut p) = prod.lock() {
                        for &sample in data {
                            let f = (sample as f32 / u32::MAX as f32) * 2.0 - 1.0;
                            p.try_push(f).ok();
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U8 => {
            let prod = producer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[u8], _: &_| {
                    if let Ok(mut p) = prod.lock() {
                        for &sample in data {
                            let f = (sample as f32 / 128.0) - 1.0;
                            p.try_push(f).ok();
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I8 => {
            let prod = producer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i8], _: &_| {
                    if let Ok(mut p) = prod.lock() {
                        for &sample in data {
                            let f = sample as f32 / i8::MAX as f32;
                            p.try_push(f).ok();
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    println!("Recording started");

    // Wait for stop signal
    while !STOP_SIGNAL.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(50));
    }

    // Stream is dropped here, stopping recording
    println!("Recording stopped");
    Ok(())
}

#[tauri::command]
pub fn stop_recording(state: State<AppState>) -> Result<AudioSamples, String> {
    if !RECORDING_ACTIVE.load(Ordering::SeqCst) {
        return Err("Not recording".to_string());
    }

    // Signal stop
    STOP_SIGNAL.store(true, Ordering::SeqCst);

    // Wait for recording thread to finish
    let mut attempts = 0;
    while RECORDING_ACTIVE.load(Ordering::SeqCst) && attempts < 100 {
        thread::sleep(std::time::Duration::from_millis(50));
        attempts += 1;
    }

    // Get recording parameters
    let (sample_rate, channels) = {
        let rec = state.recording.lock().unwrap();
        (rec.sample_rate, rec.channels)
    };

    // Collect samples from ring buffer
    let mut samples = Vec::new();
    if let Some(consumer_arc) = state.sample_consumer.lock().unwrap().take() {
        let mut consumer = consumer_arc.lock().unwrap();
        while let Some(sample) = consumer.try_pop() {
            samples.push(sample);
        }
    }

    if samples.is_empty() {
        return Err("No audio recorded".to_string());
    }

    println!(
        "Collected {} samples at {}Hz, {} channels",
        samples.len(),
        sample_rate,
        channels
    );

    // Convert to mono if stereo
    let mono_samples: Vec<f32> = if channels > 1 {
        samples
            .chunks(channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
            .collect()
    } else {
        samples
    };

    // Resample to 16kHz if needed (whisper requirement)
    let final_samples = if sample_rate != 16000 {
        resample(&mono_samples, sample_rate, 16000)
    } else {
        mono_samples
    };

    println!(
        "Final samples: {} (resampled to 16kHz, memory-only)",
        final_samples.len()
    );

    // Return samples in memory - never written to disk
    Ok(AudioSamples {
        samples: final_samples,
        sample_rate: 16000,
    })
}

#[tauri::command]
pub fn cancel_recording(state: State<AppState>) -> Result<(), String> {
    STOP_SIGNAL.store(true, Ordering::SeqCst);

    // Wait for thread to stop
    let mut attempts = 0;
    while RECORDING_ACTIVE.load(Ordering::SeqCst) && attempts < 100 {
        thread::sleep(std::time::Duration::from_millis(50));
        attempts += 1;
    }

    // Clear the consumer
    state.sample_consumer.lock().unwrap().take();

    Ok(())
}

// Simple linear resampling
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut result = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac as f32) + samples[idx + 1] * frac as f32
        } else {
            samples[idx.min(samples.len() - 1)]
        };

        result.push(sample);
    }

    result
}

#[tauri::command]
pub fn is_recording() -> bool {
    RECORDING_ACTIVE.load(Ordering::SeqCst)
}

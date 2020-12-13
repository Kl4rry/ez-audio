use std::ffi::{CStr, CString, OsStr};
use std::fs::metadata;
use std::iter::Iterator;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use std::error::Error;
use std::fmt;

static mut ID: AtomicUsize = AtomicUsize::new(0);

fn get_id() -> usize {
    unsafe { ID.fetch_add(1, Ordering::Relaxed) }
}

#[repr(C)]
struct AudioDevice {
    id: [u8; 256],
    name: *const c_char,
}

#[repr(C)]
struct AudioContext {
    context: usize,
    sound_clips: usize,
    result: bool,
    mtx: usize,
}

extern "C" {
    fn init() -> AudioContext;
    fn uninit(context: *const AudioContext);

    fn load(
        id: usize,
        context: *const AudioContext,
        path: *const c_char,
        device: *const AudioDevice,
    ) -> i32;
    fn removeSound(id: usize, context: *const AudioContext);

    fn play(id: usize, context: *const AudioContext);
    fn stop(id: usize, context: *const AudioContext);
    fn reset(id: usize, context: *const AudioContext);
    fn setVolume(id: usize, context: *const AudioContext, value: f32);
    fn getVolume(id: usize, context: *const AudioContext) -> f32;

    fn isPlaying(id: usize, context: *const AudioContext) -> bool;
    fn getDuration(id: usize, context: *const AudioContext) -> u64;

    fn getDefaultAudioDevice(context: *const AudioContext) -> AudioDevice;
    fn getAudioDevices(
        context: *const AudioContext,
        devices: *const AudioDevice,
        capacity: usize,
    ) -> usize;
    fn getAudioDeviceCount(context: &AudioContext) -> usize;
}

#[derive(Debug, Clone)]
pub enum AudioError {
    FileError,
    DecoderError,
    DeviceError,
    ContextError,
    UnknownError,
}

impl Error for AudioError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AudioError::FileError => write!(f, "unable to find file"),
            AudioError::DecoderError => write!(f, "unable to decode file"),
            AudioError::DeviceError => write!(f, "invalid device"),
            AudioError::ContextError => write!(f, "unable to initialize context"),
            AudioError::UnknownError => write!(f, "unknown error"), //this should never happen
        }
    }
}

pub fn default_output_device(context: Context) -> Device {
    Device {
        device: unsafe { getDefaultAudioDevice(&context.inner.context) },
        _context: context,
    }
}

pub struct Device {
    device: AudioDevice,
    _context: Context,
}

impl<'a> Device {
    pub fn name(&self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.device.name)
                .to_str()
                .unwrap_or("Undefined")
        }
    }
}

pub fn output_devices(context: Context) -> Devices {
    unsafe {
        let capacity = getAudioDeviceCount(&context.inner.context);
        let mut devices: Vec<AudioDevice> = Vec::with_capacity(capacity);
        let ptr = devices.as_mut_ptr();
        std::mem::forget(devices);
        let len = getAudioDevices(&context.inner.context, ptr, capacity);

        let devices = Vec::from_raw_parts(ptr, len, capacity);

        Devices { devices, context }
    }
}

pub struct Devices {
    devices: Vec<AudioDevice>,
    context: Context,
}

impl<'a> Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        let option = self.devices.pop();
        if let Some(device) = option {
            Some(Device {
                device,
                _context: self.context.clone(),
            })
        } else {
            None
        }
    }
}

struct InnerContext {
    context: AudioContext,
}

#[derive(Clone)]
pub struct Context {
    inner: Arc<InnerContext>,
}

impl Context {
    pub fn new() -> Result<Context, AudioError> {
        unsafe {
            let context = init();
            if context.result {
                Ok(Context {
                    inner: Arc::new(InnerContext { context }),
                })
            } else {
                Err(AudioError::ContextError)
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {}
}

impl Drop for InnerContext {
    fn drop(&mut self) {
        unsafe {
            uninit(&self.context);
        }
    }
}

pub struct AudioHandle {
    id: usize,
    path: PathBuf,
    context: Context,
}

impl AudioHandle {
    pub fn load<P: AsRef<Path>>(
        path: P,
        context: Context,
        device: &Device,
    ) -> Result<AudioHandle, AudioError> {
        if metadata(path.as_ref()).is_err() {
            return Err(AudioError::FileError);
        };

        unsafe {
            let id = get_id();
            let result = load(
                id,
                &context.inner.context,
                CString::new(path.as_ref().as_os_str().to_str().unwrap())
                    .unwrap()
                    .as_ptr(),
                &device.device,
            );

            match result {
                0 => Ok(AudioHandle {
                    id: id,
                    path: path.as_ref().to_path_buf(),
                    context,
                }),
                -1 => Err(AudioError::DecoderError),
                -2 => Err(AudioError::DeviceError),
                _ => Err(AudioError::UnknownError),
            }
        }
    }

    pub fn play(&self) {
        unsafe {
            play(self.id, &self.context.inner.context);
        }
    }

    pub fn stop(&self) {
        unsafe {
            stop(self.id, &self.context.inner.context);
        }
    }

    pub fn reset(&self) {
        unsafe {
            reset(self.id, &self.context.inner.context);
        }
    }

    pub fn path(&self) -> &Path {
        return &self.path;
    }

    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .unwrap_or(OsStr::new("Undefined"))
            .to_str()
            .unwrap_or("Undefined")
    }

    pub fn set_volume(&self, volume: f32) {
        unsafe {
            setVolume(self.id, &self.context.inner.context, volume);
        }
    }

    pub fn volume(&self) -> f32 {
        unsafe { getVolume(self.id, &self.context.inner.context) }
    }

    pub fn is_playing(&self) -> bool {
        unsafe { isPlaying(self.id, &self.context.inner.context) }
    }

    pub fn is_paused(&self) -> bool {
        unsafe { !isPlaying(self.id, &self.context.inner.context) }
    }

    pub fn duration(&self) -> Duration {
        unsafe { Duration::from_millis(getDuration(self.id, &self.context.inner.context)) }
    }
}

impl Drop for AudioHandle {
    fn drop(&mut self) {
        unsafe {
            removeSound(self.id, &self.context.inner.context);
        }
    }
}

/*
fn main() {
    let context = Context::init().unwrap();

    let mut clips: Vec<AudioHandle> = Vec::new();

    for _ in 0..1 {
        let clip = AudioHandle::load(
            "Genji_-_Mada_mada!.ogg",
            &context,
            &default_output_device(&context),
        )
        .unwrap();
        clips.push(clip);
    }

    let devices = output_devices(&context);

    for device in devices {
        println!("{}", device.name());
    }

    println!("{}", clips[0].duration().as_millis());

    for i in 0..1 {
        clips[i].play();
    }

    loop {}
}
*/

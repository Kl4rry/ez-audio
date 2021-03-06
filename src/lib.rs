//! ez-audio is a easy to use audio playback library that uses the C library [miniaudio](https://github.com/mackron/miniaudio) as a backend.
//! # Examples
//! ## Minimal
//! ```
//! let context = Context::new().unwrap();
//! let mut clip = AudioLoader::new("audio.mp3", context.clone())
//!     .load()
//!     .unwrap();
//! 
//! clip.play();
//! loop {}
//! ```
//! ## With on end
//! ```
//! let context = Context::new().unwrap();
//! let mut clip = AudioLoader::new("audio.mp3", context.clone())
//!     .user_data(10)
//!     .on_end(|data| {
//!         assert!(data == 10)
//!     })
//!     .load()
//!     .unwrap();
//! 
//! clip.play();
//! loop {}
//! ```
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(get_mut_unchecked)]
#![warn(missing_docs)]

use std::ffi::{CStr, CString, OsStr};
use std::fs::metadata;
use std::iter::Iterator;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use std::error::Error;
use std::fmt;

mod void;

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
    context: usize,     //pointer not real usize
    sound_clips: usize, //pointer not real usize
    result: bool,
    mtx: usize, //pointer not real usize
}

#[allow(improper_ctypes)]
extern "C" {
    fn init(end_callback: unsafe extern "C" fn(*mut InnerHandle<()>)) -> AudioContext;
    fn uninit(context: *const AudioContext);

    fn load(
        id: usize,
        context: *const AudioContext,
        path: *const c_char,
        device: *const AudioDevice,
    ) -> i32;
    fn setOuter(id: usize, context: *const AudioContext, outer: *const InnerHandle<()>);
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
    fn setAudioDevice(id: usize, context: *const AudioContext, device: *const AudioDevice);
}

/// A general purpose error.
#[derive(Debug, Clone)]
pub enum AudioError {
    /// Unable to find file.
    FileError,
    /// Unable to decode file.
    DecoderError,
    /// Error getting audio device.
    DeviceError,
    /// Error initializing backend.
    ContextError,
    /// Catch all error that should never occur.
    /// If it occurs it is a sign of undefined behavior. 
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
            AudioError::UnknownError => unreachable!(),
        }
    }
}

/// Yields default output device.
pub fn default_output_device(context: Context) -> Device {
    Device {
        device: unsafe { getDefaultAudioDevice(&context.inner.context) },
        _context: context,
    }
}

/// A handle to an audio playback device.
pub struct Device {
    device: AudioDevice,
    _context: Context,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl<'a> Device {
    /// Returns name of audio device.
    pub fn name(&self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.device.name)
                .to_str()
                .unwrap_or("Undefined")
        }
    }
}

/// Yields an iterator over all audio playback devices.
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

/// A iterator that yields audio devices.
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

#[no_mangle]
unsafe extern "C" fn end_callback(inner_handle: *mut InnerHandle<()>) {
    (*inner_handle).on_end();
}

struct InnerContext {
    context: AudioContext,
}

/// A handle to a backend context.
#[derive(Clone)]
pub struct Context {
    inner: Arc<InnerContext>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    /// Creates new backend context
    pub fn new() -> Result<Self, AudioError> {
        unsafe {
            let context = init(end_callback);
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

impl Drop for InnerContext {
    fn drop(&mut self) {
        unsafe {
            uninit(&self.context);
        }
    }
}

/// A builder that loads an audio file into memory and returns an audio playback handle.
pub struct AudioLoader<'a, T, I, P> {
    path: P,
    context: Context,
    device: Option<&'a Device>,
    volume: f32,
    on_end: Option<I>,
    user_data: T,
}

impl<'a, P> AudioLoader<'a, (), void::Void, P>
where
    P: AsRef<Path>,
{
    /// Creates a new default audio loader.
    pub fn new(path: P, context: Context) -> AudioLoader<'a, (), void::Void, P> {
        AudioLoader {
            path,
            context,
            device: None,
            volume: 1f32,
            on_end: None,
            user_data: (),
        }
    }
}

impl<'a, T, I, P> AudioLoader<'a, T, I, P>
where
    P: AsRef<Path>,
    I: 'static + FnMut(&mut T) + Send,
{
    /// Set context.
    pub fn context(mut self, context: Context) -> Self {
        self.context = context;
        self
    }

    /// Set playback device
    pub fn device(mut self, device: &'a Device) -> Self {
        self.device = Some(device);
        self
    }

    /// Set playback volume
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Destroys loader and returns a audio handle
    pub fn load(self) -> Result<AudioHandle<T>, AudioError> {
        if metadata(self.path.as_ref()).is_err() {
            return Err(AudioError::FileError);
        };

        unsafe {
            let id = get_id();
            let result = load(
                id,
                &self.context.inner.context,
                #[allow(temporary_cstring_as_ptr)]
                CString::new(self.path.as_ref().as_os_str().to_str().unwrap())
                    .unwrap()
                    .as_ptr(),
                &self
                    .device
                    .unwrap_or(&default_output_device(self.context.clone()))
                    .device,
            );

            let res = match result {
                0 => Ok(AudioHandle {
                    inner: Arc::new(InnerHandle {
                        id,
                        path: self.path.as_ref().to_path_buf(),
                        context: self.context.clone(),
                        user_data: RwLock::new(Arc::new(self.user_data)),
                        on_end: {
                            if let Some(on_end) = self.on_end {
                                Some(Mutex::new(Box::new(on_end)))
                            } else {
                                None
                            }
                        },
                    }),
                }),
                -1 => Err(AudioError::DecoderError),
                -2 => Err(AudioError::DeviceError),
                _ => Err(AudioError::UnknownError),
            };

            if res.is_ok() {
                setOuter(
                    id,
                    &self.context.inner.context,
                    Arc::as_ptr(&res.as_ref().unwrap().inner) as *const InnerHandle<()>,
                );
            }
            res
        }
    }
}

impl<'a, T, I, P0> AudioLoader<'a, T, I, P0> {
    /// Set path to file.
    pub fn path<P1: AsRef<Path>>(self, path: P1) -> AudioLoader<'a, T, I, P1> {
        AudioLoader {
            path,
            context: self.context,
            device: self.device,
            volume: self.volume,
            on_end: self.on_end,
            user_data: self.user_data,
        }
    }
}

impl<'a, T0, I, P> AudioLoader<'a, T0, I, P> {
    /// Sets custom userdata
    pub fn user_data<T1>(self, user_data: T1) -> AudioLoader<'a, T1, I, P> {
        AudioLoader {
            path: self.path,
            context: self.context,
            device: self.device,
            volume: self.volume,
            on_end: self.on_end,
            user_data,
        }
    }
}

impl<'a, T, F0: Fn(T), P> AudioLoader<'a, T, F0, P> {
    /// Sets closure to be run when the audio handle reaches the end.
    pub fn on_end<F1: FnMut(&mut T) + Send>(self, on_end: F1) -> AudioLoader<'a, T, F1, P> {
        AudioLoader {
            path: self.path,
            context: self.context,
            device: self.device,
            volume: self.volume,
            on_end: Some(on_end),
            user_data: self.user_data,
        }
    }
}

struct InnerHandle<T> {
    id: usize,
    path: PathBuf,
    context: Context,
    user_data: RwLock<Arc<T>>,
    #[allow(clippy::type_complexity)]
    on_end: Option<Mutex<Box<dyn FnMut(&mut T) + Send>>>,
}

impl<T> InnerHandle<T> {
    fn on_end(&mut self) {
        if let Some(closure) = &mut self.on_end {
            let mut refrence = self.user_data.write().unwrap();
            unsafe {
                let user_data = Arc::get_mut_unchecked(&mut refrence);
                (closure.get_mut().unwrap())(user_data);
            }
        }
    }
}

/// A handle that can be used to control audio playback.
pub struct AudioHandle<T> {
    inner: Arc<InnerHandle<T>>,
}

impl<T> AudioHandle<T> {
    /// Starts playing audio.
    pub fn play(&self) {
        unsafe {
            play(self.inner.id, &self.inner.context.inner.context);
        }
    }

    /// Pauses playback.
    pub fn stop(&self) {
        unsafe {
            stop(self.inner.id, &self.inner.context.inner.context);
        }
    }

    /// Resets to start of audio clip.
    pub fn reset(&self) {
        unsafe {
            reset(self.inner.id, &self.inner.context.inner.context);
        }
    }

    /// Returns the path used to create the handle.
    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    /// Returns name of file used to create the handle.
    pub fn name(&self) -> &str {
        self.inner
            .path
            .file_name()
            .unwrap_or_else(|| OsStr::new("Undefined"))
            .to_str()
            .unwrap_or("Undefined")
    }

    /// Sets volume.
    pub fn set_volume(&self, volume: f32) {
        unsafe {
            setVolume(self.inner.id, &self.inner.context.inner.context, volume);
        }
    }

    /// Returns volume.
    pub fn volume(&self) -> f32 {
        unsafe { getVolume(self.inner.id, &self.inner.context.inner.context) }
    }

    /// Checks if the handle is currently playing
    pub fn is_playing(&self) -> bool {
        unsafe { isPlaying(self.inner.id, &self.inner.context.inner.context) }
    }

    /// Checks if the handle is currently paused
    pub fn is_paused(&self) -> bool {
        unsafe { !isPlaying(self.inner.id, &self.inner.context.inner.context) }
    }

    /// Gets duration of audio handle
    pub fn duration(&self) -> Duration {
        unsafe {
            Duration::from_millis(getDuration(
                self.inner.id,
                &self.inner.context.inner.context,
            ))
        }
    }

    /// Sets playback device
    pub fn set_output_device(&self, device: &Device) {
        unsafe {
            setAudioDevice(
                self.inner.id,
                &self.inner.context.inner.context,
                &device.device,
            )
        }
    }

    /// Sets userdata.
    pub fn set_user_data(&mut self, data: T) {
        unsafe {
            *Arc::get_mut_unchecked(&mut *self.inner.user_data.write().unwrap()) = data;
        }
    }

    /// Modifes userdata using closure.
    pub fn modify_user_data<I: FnMut(&mut T)>(&self, mut closure: I) {
        let mut refrence = self.inner.user_data.write().unwrap();
        unsafe {
            let user_data = Arc::get_mut_unchecked(&mut refrence);
            closure(user_data);
        }
    }
}

impl<T> Drop for AudioHandle<T> {
    fn drop(&mut self) {
        unsafe {
            removeSound(self.inner.id, &self.inner.context.inner.context);
        }
    }
}

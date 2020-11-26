use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsStr;
use std::fs::metadata;
use std::os::raw::c_char;
use std::path::Path;
use std::path::PathBuf;
use std::iter::Iterator;

use std::error::Error;
use std::fmt;

static mut NEXT_ID: usize = 0;
static mut FREE_ID: Vec<usize> = Vec::new();

fn get_id() -> usize {
    unsafe {
        FREE_ID.pop().unwrap_or_else(||{
            let id = NEXT_ID;
            NEXT_ID += 1;
            id
        })
    }
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
}

extern "C" {
    fn init() -> AudioContext;
    fn uninit(context: &AudioContext);

    fn load(
        id: usize,
        context: &AudioContext,
        path: *const c_char,
        device: *const AudioDevice,
    ) -> i32;
    fn remove(id: usize, context: &AudioContext);

    fn play(id: usize, context: &AudioContext);
    fn stop(id: usize, context: &AudioContext);
    fn reset(id: usize, context: &AudioContext);

    fn getDefaultAudioDevice(context: &AudioContext) -> AudioDevice;
    fn getAudioDevices(context: &AudioContext, devices: *const AudioDevice, capacity: usize) -> usize;
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

pub fn default_output_device(context: &Context) -> Device {
    Device {
        device: unsafe { getDefaultAudioDevice(&context.context) },
    }
}

pub struct Device {
    device: AudioDevice,
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

pub fn output_devices(context: &Context) -> Devices {
    unsafe {
        let capacity = getAudioDeviceCount(&context.context);
        let mut devices: Vec<AudioDevice> = Vec::with_capacity(capacity);
        let mut ptr = devices.as_mut_ptr();
        let len = getAudioDevices(&context.context, ptr, capacity);
        std::mem::forget(devices);

        let devices = Vec::from_raw_parts(ptr, len, capacity);
        
        Devices {
            devices,
        }
    }
}

pub struct Devices {
    devices: Vec<AudioDevice>,
}

impl<'a> Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        let option = self.devices.pop();
        if let Some(device) = option {
            Some(Device{
                device,
            })
        } else {
            None
        }
    }
}

pub struct Context {
    context: AudioContext,
}

impl Context {
    pub fn init() -> Result<Context, AudioError> {
        unsafe {
            let context = init();
            if context.result {
                Ok(Context { context })
            } else {
                Err(AudioError::ContextError)
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            uninit(&self.context);
        }
    }
}

pub struct AudioHandle<'a> {
    id: usize,
    path: PathBuf,
    context: &'a Context,
}

impl<'a> AudioHandle<'a> {
    fn load<P: AsRef<Path>>(
        path: P,
        context: &'a Context,
        device: Device,
    ) -> Result<AudioHandle, AudioError> {
        if metadata(path.as_ref()).is_err() {
            return Err(AudioError::FileError);
        };

        unsafe {
            let id = get_id();
            let result = load(
                id,
                &context.context,
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

    fn play(&self) {
        unsafe {
            play(self.id, &self.context.context);
        }
    }

    fn stop(&self) {
        unsafe {
            stop(self.id, &self.context.context);
        }
    }

    fn reset(&self) {
        unsafe {
            reset(self.id, &self.context.context);
        }
    }

    fn path(&self) -> &Path {
        return &self.path;
    }

    fn name(&self) -> &str {
        self.path
            .file_name()
            .unwrap_or(OsStr::new("Undefined"))
            .to_str()
            .unwrap_or("Undefined")
    }
}

impl<'a> Drop for AudioHandle<'a> {
    fn drop(&mut self) {
        unsafe {
            remove(self.id, &self.context.context);
        }
    }
}

fn main() {
    let context = Context::init().unwrap();
    let clip = AudioHandle::load(
        "Genji_-_Mada_mada!.ogg",
        &context,
        default_output_device(&context),
    )
    .unwrap();

    let devices = output_devices(&context);

    for device in devices {
        println!("{}", device.name());
    }

    clip.play();
    loop {}
}

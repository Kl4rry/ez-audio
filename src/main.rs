use std::os::raw::c_char;
use std::ffi::CString;
use std::ffi::CStr;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::path::Path;
use std::fs::metadata;

use std::error::Error;
use std::fmt;

static mut NEXT_ID: usize = 0;

fn get_id() -> usize {
    unsafe {
        let id = NEXT_ID;
        NEXT_ID += 1;
        id
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

    fn load(id: usize, context: &AudioContext, path: *const c_char, device: *const AudioDevice) -> i32;
    fn remove(id: usize, context: &AudioContext);

    fn play(id: usize, context: &AudioContext);
    fn stop(id: usize, context: &AudioContext);
    fn reset(id: usize, context: &AudioContext);

    fn getDefaultAudioDevice(context: &AudioContext) -> AudioDevice;
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
            AudioError::UnknownError => write!(f, "unknown error"),//this should never happen
        }
    }
}

pub fn default_output_device(context: &Context) -> Device {
    Device {
        device: unsafe{getDefaultAudioDevice(&context.context)},
    }
}

pub struct Device {
    device: AudioDevice,
}

impl<'a> Device {
    pub fn name(&self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.device.name).to_str().unwrap_or("Undefined")
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
                Ok(Context{
                    context,
                })
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

pub struct AudioBuffer<'a> {
    id: usize,
    path: PathBuf,
    context: &'a Context,
}

impl<'a> AudioBuffer<'a> {
    fn load<P: AsRef<Path>>(path: P, context: &'a Context, device: Device) -> Result<AudioBuffer, AudioError> {
        if metadata(path.as_ref()).is_err() {
            return Err(AudioError::FileError)
        };

        unsafe{
            let id = get_id();
            let result = load(id, &context.context, CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap().as_ptr(), &device.device);

            match result {
                0 => {
                    Ok(AudioBuffer{
                        id:  id,
                        path: path.as_ref().to_path_buf(),
                        context,
                    })
                }
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
        self.path.file_name().unwrap_or(OsStr::new("Undefined")).to_str().unwrap_or("Undefined")
    }
}

impl<'a> Drop for AudioBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            remove(self.id, &self.context.context);
        }
    }
}

fn main() {
    let context = Context::init().unwrap();
    let clip = AudioBuffer::load("Genji_-_Mada_mada!.ogg", &context, default_output_device(&context)).unwrap();
    clip.play();
    loop{}
}

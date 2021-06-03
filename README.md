# ez-audio
[![Crates.io](https://img.shields.io/crates/v/ez_audio.svg)](https://crates.io/crates/ez_audio)
[![Docs.rs](https://docs.rs/ez_audio/badge.svg)](https://docs.rs/ez_audio)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)  
ez-audio is a easy to use audio playback library that uses the C library [miniaudio](https://github.com/mackron/miniaudio) as a backend.  
Currently only compiles on nightly and a C++ compiler is required as it depends on the [cc crate](https://crates.io/crates/cc).

## Supported Codecs
- MP3  
- WAV  
- Vorbis  
- Flac  


# Examples
## Minimal
```rust
    let context = Context::new().unwrap();
    let mut clip = AudioLoader::new("audio.mp3", context.clone())
        .load()
        .unwrap();

    clip.play();
    // loop forever to stop handle from being dropped
    loop {}
```

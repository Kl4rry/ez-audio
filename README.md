# ez-audio
ez-audio is a easy to use audio playback library that uses the C library [miniaudio](https://github.com/mackron/miniaudio) as a backend.

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
    loop {}
```
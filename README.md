# ez-audio
A easy to use audio playback library

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
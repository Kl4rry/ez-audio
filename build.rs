fn main() {
    cc::Build::new()
        .cpp(true)
        .file("cc/AudioInterface.cc")
        .file("cc/AudioPlayer.cc")
        .compile("libezaudio.a");
}

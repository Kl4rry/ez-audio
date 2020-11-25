#define STB_VORBIS_HEADER_ONLY
#include "stb_vorbis.c"
#define MINIAUDIO_IMPLEMENTATION
#include "AudioPlayer.h"
#undef STB_VORBIS_HEADER_ONLY
#include "stb_vorbis.c"

#include <iostream>

//https://miniaud.io/docs/examples/simple_mixing.html
//TODO add mixing
//give every context one device
//probably separeating device and context to match api
//this is clearly worng but it do work for now

void data_callback(ma_device* device, void* output, const void*, ma_uint32 framesToRead){
	SoundClip* clip = (SoundClip*)device->pUserData;
	if(&clip->decoder == NULL){
			return;
	}
	ma_uint64 framesRead = ma_decoder_read_pcm_frames(&clip->decoder, output, framesToRead);
	if(framesRead < framesToRead){
		float oldVolume = device->masterVolumeFactor;
		device->masterVolumeFactor = 0;
		ma_decoder_seek_to_pcm_frame(&clip->decoder, 0);
		resetDevice(device, clip, oldVolume);
	}
}

void resetDevice(ma_device* device, SoundClip* clip, float const& oldVolume){
	std::thread t{[device, clip, oldVolume](){
		
		std::lock_guard<std::mutex> lock(clip->mtx);
		ma_device_stop(device);
		ma_decoder_seek_to_pcm_frame(&clip->decoder, 0);
		device->masterVolumeFactor = oldVolume;
		//call end callback
		
	}};
	t.detach();
}

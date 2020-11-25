#pragma once
#include "miniaudio.h"
#include <unordered_map>
#include <thread>
#include <functional>
#include <atomic>
#include <mutex>
#include <array>

struct AudioContext;

struct AudioDevice{
	ma_device_id id;
	const char* name;
};

struct SoundClip{
	ma_device device;
	ma_decoder decoder;
	ma_device_config deviceConfig;
	size_t id;
	std::mutex mtx;
	AudioDevice* audioDevice;
	AudioContext* context;
};

struct AudioContext{
	ma_context* context;
	std::unordered_map<size_t, SoundClip*>* soundClips;
	bool result;
};

void data_callback(ma_device* device, void* output, const void* input, ma_uint32 frameCount);
void resetDevice(ma_device* device, SoundClip* clip, float const& oldVolume);

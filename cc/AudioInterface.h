#pragma once
#include "AudioPlayer.h"
#include <vector>
#include <iostream>
#include <functional>
#include <chrono>

extern "C" AudioContext init();

extern "C" void uninit(AudioContext* context);

extern "C" void setVolume(size_t id, AudioContext* context, float value);

extern "C" void play(size_t id, AudioContext* context);

extern "C" void reset(size_t id, AudioContext* context);

extern "C" void stop(size_t id, AudioContext* context);

extern "C" int load(size_t id, AudioContext* context, const char* path, AudioDevice* device);

extern "C" void removeSound(size_t id, AudioContext* context);

//extern "C" std::vector<AudioDevice> getDeviceList();

extern "C" void setAudioDevice(size_t id, AudioContext* context, AudioDevice* device);

extern "C" AudioDevice getDefaultAudioDevice(AudioContext* context);

extern "C" uint64_t getDuration(size_t id, AudioContext* context);

extern "C" bool isPlaying(size_t id, AudioContext* context);

#pragma once
#include "AudioPlayer.h"
#include <vector>
#include <iostream>
#include <functional>
#include <chrono>
#include <mutex>

extern "C" AudioContext init();

extern "C" void uninit(AudioContext* context);

extern "C" void setVolume(size_t id, AudioContext* context, float value);

extern "C" float getVolume(size_t id, AudioContext* context);

extern "C" void play(size_t id, AudioContext* context);

extern "C" void reset(size_t id, AudioContext* context);

extern "C" void stop(size_t id, AudioContext* context);

extern "C" int load(size_t id, AudioContext* context, const char* path, AudioDevice* device);

extern "C" void removeSound(size_t id, AudioContext* context);

extern "C" size_t getAudioDevices(AudioContext* context, AudioDevice* devices, size_t capacity);

extern "C" size_t getAudioDeviceCount(AudioContext* context);

extern "C" void setAudioDevice(size_t id, AudioContext* context, AudioDevice* device);

extern "C" AudioDevice getDefaultAudioDevice(AudioContext* context);

extern "C" uint64_t getDuration(size_t id, AudioContext* context);

extern "C" bool isPlaying(size_t id, AudioContext* context);

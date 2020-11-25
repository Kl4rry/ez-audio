#include "AudioInterface.h"

extern "C" AudioContext init(){
	ma_context* context = new ma_context();
	if(ma_context_init(NULL, 0, NULL, context) != MA_SUCCESS){
		std::cout << "Failed to initialize context" << std::endl;
		delete context;
		return AudioContext{nullptr, nullptr, false};
	}

	return AudioContext{context, new std::unordered_map<size_t, SoundClip*>, true};
}

extern "C" void uninit(AudioContext* context){
	for(std::pair<size_t, SoundClip*> clip : *(context->soundClips)){
		ma_decoder_uninit(&clip.second->decoder);
		ma_device_uninit(&clip.second->device);
		delete clip.second;
	}
	delete context->soundClips;
	ma_context_uninit(context->context);
	delete context;
}

void setVolume(size_t id, AudioContext* context, float value){
	context->soundClips->at(id)->device.masterVolumeFactor = value;
}

extern "C" void play(size_t id, AudioContext* context){
	if(!ma_device_is_started(&context->soundClips->at(id)->device)){
		if(ma_device_start(&context->soundClips->at(id)->device) != MA_SUCCESS){
			std::cout << "Failed to start playback: " << std::endl;
		}
	}
}

extern "C" void reset(size_t id, AudioContext* context){
	ma_device_stop(&context->soundClips->at(id)->device);
	context->soundClips->at(id)->device.masterVolumeFactor = 0;
	ma_decoder_seek_to_pcm_frame(&context->soundClips->at(id)->decoder, 0);
}

extern "C" void stop(size_t id, AudioContext* context){
	ma_device_stop(&context->soundClips->at(id)->device);
}

extern "C" int load(size_t id, AudioContext* context, const char* path, AudioDevice* device){
	SoundClip* soundClip = new SoundClip;
	soundClip->id = id;
	soundClip->audioDevice = device;

	//creating and configuring decoder
	if(ma_decoder_init_file(path, NULL, &soundClip->decoder) != MA_SUCCESS){
		ma_decoder_uninit(&soundClip->decoder);
		delete soundClip;
		return -1;
	}

	//configure device
	soundClip->deviceConfig = ma_device_config_init(ma_device_type_playback);
	soundClip->deviceConfig.playback.format   = soundClip->decoder.outputFormat;
	soundClip->deviceConfig.playback.channels = soundClip->decoder.outputChannels;
	soundClip->deviceConfig.sampleRate        = soundClip->decoder.outputSampleRate;
	soundClip->deviceConfig.dataCallback      = data_callback;
	soundClip->deviceConfig.pUserData         = soundClip;

	soundClip->deviceConfig.playback.pDeviceID = &device->id;

	if(ma_device_init(context->context, &soundClip->deviceConfig, &soundClip->device) != MA_SUCCESS){
		std::cout << "Failed to open playback device" << std::endl;
		ma_decoder_uninit(&soundClip->decoder);
		delete soundClip;
		return -2;
	}

	soundClip->device.masterVolumeFactor = 1;

	context->soundClips->insert({id, soundClip});

	return 0;
}

extern "C" void removeSound(size_t id, AudioContext* context){
	ma_device_uninit(&context->soundClips->at(id)->device);
	ma_decoder_uninit(&context->soundClips->at(id)->decoder);
	delete context->soundClips->at(id);
	context->soundClips->erase(id);
}


/*extern "C" void getDeviceList(AudioDevice devices[], size_t deviceCount){
	ma_device_info* playbackDeviceInfos;
	ma_uint32 playbackDeviceCount;
	std::vector<AudioDevice> deviceList{};
	if(ma_context_get_devices(&context, &playbackDeviceInfos, &playbackDeviceCount, NULL, NULL) != MA_SUCCESS){
		std::cout << "Failed to retrieve device information" << std::endl;
	}
	*devices = (AudioDevice)malloc(playbackDeviceCount * sizeof(AudioDevice));
	//*devices = AudioDevice[playbackDeviceCount];
	for (ma_uint32 i{0}; i < playbackDeviceCount; ++i) {
		deviceList.push_back(AudioDevice{
			playbackDeviceInfos[i].id,
			playbackDeviceInfos[i].name
		});
	}
	return deviceList;
}*/

extern "C" void setAudioDevice(size_t id, AudioContext* context, AudioDevice* device){
	ma_device_info* playbackDeviceInfos;
	ma_uint32 playbackDeviceCount;
	if(ma_context_get_devices(context->context, &playbackDeviceInfos, &playbackDeviceCount, NULL, NULL) != MA_SUCCESS){
		std::cout << "Failed to retrieve device information" << std::endl;
	}
	context->soundClips->at(id)->audioDevice = device;
		
	ma_device_uninit(&context->soundClips->at(id)->device);
	ma_device_init(context->context, &context->soundClips->at(id)->deviceConfig, &context->soundClips->at(id)->device);
}

extern "C" AudioDevice getDefaultAudioDevice(AudioContext* context){
	ma_device_info* playbackDeviceInfos;
	ma_uint32 playbackDeviceCount;
	if(ma_context_get_devices(context->context, &playbackDeviceInfos, &playbackDeviceCount, NULL, NULL) != MA_SUCCESS){
		std::cout << "Failed to retrieve device information" << std::endl;
	}
	for(size_t i{0}; i<playbackDeviceCount; ++i){
		if(playbackDeviceInfos[i].isDefault) {
			return AudioDevice{
				playbackDeviceInfos[i].id,
				playbackDeviceInfos[i].name
			};
		}
	}
	return AudioDevice{
		playbackDeviceInfos[0].id,
		playbackDeviceInfos[0].name
	};
}

extern "C" uint64_t getDuration(size_t id, AudioContext* context){
	uint64_t sampleRate{context->soundClips->at(id)->device.sampleRate};
	uint64_t duration{ma_decoder_get_length_in_pcm_frames(&context->soundClips->at(id)->decoder)};
	return duration/(sampleRate/1000);
}

extern "C" bool isPlaying(size_t id, AudioContext* context){
  	return ma_device_is_started(&context->soundClips->at(id)->device);
}

#include "AudioInterface.h"
#include <iostream>

using namespace std;

int main(){
    auto context = init();
    if(context.result){
        cout <<  "hello" << endl;
        AudioDevice* device = new AudioDevice(getDefaultAudioDevice(&context));
        load(1, &context, "../slam.mp3", device);
        play(1, &context);
        cout << getDuration(1, &context) << endl;
        cout << sizeof(device->id) << endl;
        cout << getDefaultAudioDevice(&context).name << endl;
    }

    

    getchar();
    uninit(&context);
}



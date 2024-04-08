extern crate winapi;
use winapi::shared::winerror::*;
use winapi::um::combaseapi::*;
use winapi::um::endpointvolume::*;
use winapi::um::mmdeviceapi::*;
use std::ptr::null_mut;
use crate::toast;
/*###############################################################################
Windows Volume Setup
    Personally, I use SteelSeries Sonar for volume control, but I am 
###############################################################################*/
// Define IID_IMMDeviceEnumerator manually
// This is the interface ID for IMMDeviceEnumerator
#[allow(non_upper_case_globals)]
const IID_IMMDeviceEnumerator: winapi::shared::guiddef::GUID = winapi::shared::guiddef::GUID {
    Data1: 0xA95664D2,
    Data2: 0x9614,
    Data3: 0x4F35,
    Data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
};

// Define IID_IAudioEndpointVolume manually
// This is the interface ID for IAudioEndpointVolume
#[allow(non_upper_case_globals)]
const IID_IAudioEndpointVolume: winapi::shared::guiddef::GUID = winapi::shared::guiddef::GUID {
    Data1: 0x5CDF2C82,
    Data2: 0x841E,
    Data3: 0x4546,
    Data4: [0x97, 0x22, 0x0C, 0xF7, 0x40, 0x78, 0x22, 0x9A],
};


// Function to initialize COM library and get the audio endpoint volume interface
pub fn get_audio_endpoint_volume() -> Result<*mut IAudioEndpointVolume, HRESULT> {
    unsafe {
        CoInitializeEx(null_mut(), COINITBASE_MULTITHREADED);
        let mut device_enumerator: *mut IMMDeviceEnumerator = null_mut();
        let hr = CoCreateInstance(&CLSID_MMDeviceEnumerator, null_mut(), CLSCTX_ALL, &IID_IMMDeviceEnumerator, &mut device_enumerator as *mut _ as *mut _);
        if SUCCEEDED(hr) {
            let mut default_device: *mut IMMDevice = null_mut();
            (*device_enumerator).GetDefaultAudioEndpoint(eRender, eConsole, &mut default_device);
            let mut endpoint_volume: *mut IAudioEndpointVolume = null_mut();
            (*default_device).Activate(&IID_IAudioEndpointVolume, CLSCTX_ALL, null_mut(), &mut endpoint_volume as *mut _ as *mut _);
            (*device_enumerator).Release();
            (*default_device).Release();
            Ok(endpoint_volume)
        } else {
            Err(hr)
        }
    }
}

pub fn set_system_volume(volume: f32) {
    match get_audio_endpoint_volume() {
        Ok(endpoint_volume) => {
            if !endpoint_volume.is_null() {
                unsafe {
                    let hr = (*endpoint_volume).SetMasterVolumeLevelScalar(volume, null_mut());
                    (*endpoint_volume).Release();
                    if FAILED(hr) {
                        toast::show_toast("Volume Control Failed", &format!("Failed to set system volume: HRESULT {}", hr));
                    }
                }
            } else {
                toast::show_toast("Volume Control Failed", "Failed to get audio endpoint volume");
            }
        },
        Err(hr) => {
            toast::show_toast("Volume Control Failed", &format!("Failed to get audio endpoint volume: HRESULT {}", hr));
        },
    }
}

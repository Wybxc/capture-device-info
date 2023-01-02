use capture_device_info::dshow::capture_devices;
use capture_device_info::CaptureDeviceInfo;

fn main() {
    for device in capture_devices().unwrap() {
        println!("name: {}", device.name());
        println!("description: {}", device.description());
        println!("orientation: {:?}", device.orientation());
        println!("position: {:?}", device.position());
        println!("resolutions: {:?}", device.resolutions());
        println!();
    }
}

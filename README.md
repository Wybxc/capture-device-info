# capture-device-info

List information of capture devices.

## Usage

Means to get device information varies by platform.
Choose the appropriate way for your platform by enabling the corresponding feature.

| Feature | Platform | Description         |
| ------- | -------- | ------------------- |
| `dshow` | Windows  | Use DirectShow API. |

Each feature enables the corresponding sub-module,
which exposes the `capture_devices` method.
The function returns a iterator of `CaptureDeviceInfo`s.

The `CaptureDeviceInfo` trait provides methods to get device information,
such as name, description, and supported resolutions.
See the documentation of the trait for details.

The order of the devices is guaranteed to be consistent with OpenCV's `cv::VideoCapture::new`.

Depending on the implementation, some information may be unavailable.

## Examples

### DirectShow

```rust
use capture_device_info::CaptureDeviceInfo;
// To use DirectShow API, enable the `dshow` feature.
use capture_device_info::dshow::capture_devices;

for device in capture_devices().unwrap() {
    println!("name: {}", device.name());
    println!("description: {}", device.description());
    println!("orientation: {:?}", device.orientation());
    println!("position: {:?}", device.position());
    println!("resolutions: {:?}", device.resolutions());
    println!();
}
```

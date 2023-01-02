//! List information of capture devices.
//!
//! ## Usage
//!
//! Means to get device information varies by platform.
//! Choose the appropriate way for your platform by enabling the corresponding feature.
//!
//! | Feature | Platform | Description |
//! | ------- | -------- | ----------- |
//! | `dshow` | Windows  | Use DirectShow API. |
//!
//! Each feature enables the corresponding sub-module,
//! which exposes the `capture_devices` method.
//! The function returns a iterator of [`CaptureDeviceInfo`]s.
//!
//! The [`CaptureDeviceInfo`] trait provides methods to get device information,
//! such as name, description, and supported resolutions.
//! See the documentation of the trait for details.
//!
//! The order of the devices is guaranteed to be consistent with OpenCV's `cv::VideoCapture::new`.
//!
//! Depending on the implementation, some information may be unavailable.
//!
//! ## Examples
//!
//! ### DirectShow
//!
//! ```rust
//! use capture_device_info::CaptureDeviceInfo;
//! // To use DirectShow API, enable the `dshow` feature.
//! # #[cfg(feature = "dshow")]
//! use capture_device_info::dshow::capture_devices;
//!
//! # #[cfg(feature = "dshow")]
//! # fn main() {
//! for device in capture_devices().unwrap() {
//!     println!("name: {}", device.name());
//!     println!("description: {}", device.description());
//!     println!("orientation: {:?}", device.orientation());
//!     println!("position: {:?}", device.position());
//!     println!("resolutions: {:?}", device.resolutions());
//!     println!();
//! }
//! # }
//! ```

#![deny(missing_docs)]
#![deny(warnings)]

use std::hash::{Hash, Hasher};

#[cfg(feature = "dshow")]
pub mod dshow;

/// The information of a capture device.
///
/// Depending on the implementation, some information may be unavailable.
pub trait CaptureDeviceInfo {
    /// Returns the device name of the camera.
    ///
    /// This is a unique ID to identify the camera and may not be human-readable.
    ///
    /// For virtual cameras, the value may be meaningless, thus it is not unique.
    fn name(&self) -> &str;

    /// Returns the human-readable description of the camera.
    fn description(&self) -> &str;

    /// Returns the physical orientation of the camera sensor.
    ///
    /// The value is the orientation angle (clockwise, in steps of 90 degrees)
    /// of the camera sensor in relation to the display in its natural orientation.
    ///
    /// You can show the camera image in the correct orientation by rotating it
    /// by this value in the anti-clockwise direction.
    ///
    /// The value may be unspecified, in which case [`None`] is returned.
    fn orientation(&self) -> Option<i32>;

    /// Returns the physical position of the camera on the hardware system.
    ///
    /// The value may be unspecified, in which case [`None`] is returned.
    fn position(&self) -> Option<CaptureDevicePosition>;

    /// Returns the supported resolutions of the camera.
    ///
    /// The value may be unspecified, in which case an empty vector is returned.
    fn resolutions(&self) -> &[CaptureDeviceResolution];
}

/// The physical position of the camera on the hardware system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptureDevicePosition {
    /// The camera is on the front of the device.
    Back,
    /// The camera is on the back of the device.
    Front,
}

/// The resolution of a capture device.
#[derive(Debug, Clone)]
pub struct CaptureDeviceResolution {
    /// Width of a frame.
    pub width: u32,
    /// Height of a frame.
    pub height: u32,
    /// Frame rate (per second). Rounded to 0.01.
    pub frame_rate: f64,
}

/// The time unit used for frame rate (10ms).
const FRAME_RATE_TIME_UNIT_PER_SECOND: f64 = 1e2;

impl CaptureDeviceResolution {
    fn frame_rate_by_units(&self) -> i64 {
        (self.frame_rate * FRAME_RATE_TIME_UNIT_PER_SECOND) as i64
    }
}

impl PartialEq for CaptureDeviceResolution {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.frame_rate_by_units() == other.frame_rate_by_units()
    }
}

impl Eq for CaptureDeviceResolution {}

impl Hash for CaptureDeviceResolution {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
        self.frame_rate_by_units().hash(state);
    }
}

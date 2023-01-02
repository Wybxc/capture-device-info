//! Get capture devices from DirectShow.
//!
//! See: [MSDN](https://learn.microsoft.com/en-us/windows/win32/directshow/selecting-a-capture-device)

use std::collections::HashSet;
use std::ptr::NonNull;

use windows::core::*;
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Media::DirectShow::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Ole::*;

use crate::{CaptureDeviceInfo, CaptureDevicePosition, CaptureDeviceResolution};

/// Capture device information from DirectShow.
pub struct DirectShowCaptureDevice {
    name: String,
    description: String,
    resolution: Vec<CaptureDeviceResolution>,
}

impl CaptureDeviceInfo for DirectShowCaptureDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn orientation(&self) -> Option<i32> {
        None
    }

    fn position(&self) -> Option<CaptureDevicePosition> {
        None
    }

    fn resolutions(&self) -> &[CaptureDeviceResolution] {
        &self.resolution
    }
}

struct MonikerIterator {
    enum_moniker: IEnumMoniker,
}

impl MonikerIterator {
    pub fn enumerate_devices() -> Result<Self> {
        let enum_moniker = unsafe {
            let mut enum_moniker = None;
            let dev_enum: ICreateDevEnum = CoCreateInstance(
                &CLSID_SystemDeviceEnum,
                InParam::null(),
                CLSCTX_INPROC_SERVER,
            )?;
            dev_enum.CreateClassEnumerator(
                &CLSID_VideoInputDeviceCategory,
                &mut enum_moniker,
                0,
            )?;
            enum_moniker
                .ok_or_else(|| Error::new(VFW_E_NOT_FOUND, "No video input device found".into()))?
        };

        Ok(Self { enum_moniker })
    }
}

impl Iterator for MonikerIterator {
    type Item = IMoniker;

    fn next(&mut self) -> Option<Self::Item> {
        let mut moniker = [None; 1];
        let hr = unsafe { self.enum_moniker.Next(&mut moniker, None) };
        if hr.is_ok() {
            moniker.into_iter().next().unwrap()
        } else {
            None
        }
    }
}

struct PinIterator {
    enum_pins: IEnumPins,
}

impl PinIterator {
    pub fn enumerate_pins(moniker: &IMoniker) -> Result<Self> {
        let enum_pins = unsafe {
            let mut filter = None;
            moniker.BindToObject(
                InParam::null(),
                InParam::null(),
                &IBaseFilter::IID,
                &mut filter as *mut _ as *mut _,
            )?;
            let filter: IBaseFilter = filter.ok_or_else(|| {
                Error::new(
                    VFW_E_NOT_FOUND,
                    "Video device does not support IBaseFilter".into(),
                )
            })?;
            filter.EnumPins()?
        };
        Ok(Self { enum_pins })
    }
}

impl Iterator for PinIterator {
    type Item = IPin;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pin = [None; 1];
        let hr = unsafe { self.enum_pins.Next(&mut pin, None) };
        if hr.is_ok() {
            pin.into_iter().next().unwrap()
        } else {
            None
        }
    }
}

struct MediaType {
    mt: NonNull<AM_MEDIA_TYPE>,
    _phantom: std::marker::PhantomData<AM_MEDIA_TYPE>,
}

impl MediaType {
    fn new(mt: NonNull<AM_MEDIA_TYPE>) -> Self {
        Self {
            mt,
            _phantom: std::marker::PhantomData,
        }
    }

    fn as_ref(&self) -> &AM_MEDIA_TYPE {
        unsafe { self.mt.as_ref() }
    }

    fn frame_rate(&self) -> Option<f64> {
        let mt = self.as_ref();
        let fps = if mt.formattype == FORMAT_VideoInfo {
            let vi = unsafe { &*(mt.pbFormat as *const VIDEOINFOHEADER) };
            let fps = 10_000_000.0 / vi.AvgTimePerFrame as f64;
            Some(fps)
        } else if mt.formattype == FORMAT_VideoInfo2 {
            let vi = unsafe { &*(mt.pbFormat as *const VIDEOINFOHEADER2) };
            let fps = 10_000_000.0 / vi.AvgTimePerFrame as f64;
            Some(fps)
        } else {
            None
        };
        fps.map(|fps| (fps * 100.0).round() / 100.0)
    }

    fn bitmap_info(&self) -> Option<&BITMAPINFOHEADER> {
        let mt = self.as_ref();
        if mt.formattype == FORMAT_VideoInfo {
            let vi = unsafe { &*(mt.pbFormat as *const VIDEOINFOHEADER) };
            let bi = &vi.bmiHeader;
            Some(bi)
        } else if mt.formattype == FORMAT_VideoInfo2 {
            let vi = unsafe { &*(mt.pbFormat as *const VIDEOINFOHEADER2) };
            let bi = &vi.bmiHeader;
            Some(bi)
        } else {
            None
        }
    }
}

impl Drop for MediaType {
    fn drop(&mut self) {
        // see: https://learn.microsoft.com/en-us/windows/win32/directshow/deletemediatype
        unsafe {
            // FreeMediaType
            let mt = self.mt.as_mut();
            if !mt.pbFormat.is_null() {
                CoTaskMemFree(Some(mt.pbFormat as *mut _));
                mt.cbFormat = 0;
                mt.pbFormat = std::ptr::null_mut();
            }
            drop(mt.pUnk.take());

            // DeleteMediaType
            let mt = self.mt.as_ptr();
            CoTaskMemFree(Some(mt as *mut _));
        }
    }
}

struct MediaTypeIterator {
    enum_media_types: IEnumMediaTypes,
}

impl MediaTypeIterator {
    pub fn enumerate_media_types(pin: &IPin) -> Result<Self> {
        let enum_media_types = unsafe { pin.EnumMediaTypes()? };
        Ok(Self { enum_media_types })
    }
}

impl Iterator for MediaTypeIterator {
    type Item = MediaType;

    fn next(&mut self) -> Option<Self::Item> {
        let mut media_type = [std::ptr::null_mut(); 1];
        let hr = unsafe { self.enum_media_types.Next(&mut media_type, None) };
        if hr.is_ok() {
            let mt_ptr = media_type.into_iter().next().unwrap();
            Some(MediaType::new(NonNull::new(mt_ptr)?))
        } else {
            None
        }
    }
}

/// Get capture devices from DirectShow.
///
/// # Examples
///
/// ```rust
/// use capture_device_info::CaptureDeviceInfo;
/// // To use DirectShow API, enable the `dshow` feature.
/// # #[cfg(feature = "dshow")]
/// use capture_device_info::dshow::capture_devices;
///
/// # #[cfg(feature = "dshow")]
/// # fn main() {
/// for device in capture_devices().unwrap() {
///     // ...
/// }
/// # }
/// ```
pub fn capture_devices() -> Result<impl Iterator<Item = DirectShowCaptureDevice>> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED)? };

    let devices: Vec<_> = MonikerIterator::enumerate_devices()?
        .map(|moniker| {
            // get property bag
            let mut prop_bag: Option<IPropertyBag> = None;
            unsafe {
                moniker.BindToStorage(
                    InParam::null(),
                    InParam::null(),
                    &IPropertyBag::IID,
                    &mut prop_bag as *mut _ as *mut _,
                )?;
            }
            let prop_bag = prop_bag.unwrap();

            // initialize variant
            let mut variant = Default::default();

            // get description from "Description" or "FriendlyName"
            let description = unsafe {
                VariantInit(&mut variant);
                if prop_bag
                    .Read(&"Description".into(), &mut variant, InParam::null())
                    .is_err()
                {
                    prop_bag.Read(&"FriendlyName".into(), &mut variant, InParam::null())?;
                }
                // see: https://github.com/microsoft/windows-rs/issues/539
                let desc = variant.Anonymous.Anonymous.Anonymous.bstrVal.to_string();
                VariantClear(&mut variant)?;
                desc
            };

            // get device path from "DevicePath"
            let device_path = unsafe {
                VariantInit(&mut variant);
                if prop_bag
                    .Read(&"DevicePath".into(), &mut variant, InParam::null())
                    .is_ok()
                {
                    let path = variant.Anonymous.Anonymous.Anonymous.bstrVal.to_string();
                    VariantClear(&mut variant)?;
                    path
                } else {
                    String::new()
                }
            };

            // get pins
            let mut resolution = HashSet::new();
            for pin in PinIterator::enumerate_pins(&moniker)? {
                // filter output pins
                let pin_info = unsafe { pin.QueryPinInfo() }?;
                if pin_info.dir != PINDIR_OUTPUT {
                    continue;
                }

                // get media types
                for media_type in MediaTypeIterator::enumerate_media_types(&pin)? {
                    if let Some(bi) = media_type.bitmap_info() {
                        let width = bi.biWidth as u32;
                        let height = bi.biHeight.unsigned_abs();
                        let frame_rate = media_type.frame_rate().unwrap();
                        resolution.insert(CaptureDeviceResolution {
                            width,
                            height,
                            frame_rate,
                        });
                    }
                }
            }
            let mut resolution: Vec<_> = resolution.into_iter().collect();
            resolution.sort_unstable_by(|a, b| {
                let a = (a.width as f64) * (a.height as f64) * a.frame_rate;
                let b = (b.width as f64) * (b.height as f64) * b.frame_rate;
                b.partial_cmp(&a).unwrap()
            });

            Ok(DirectShowCaptureDevice {
                name: device_path,
                description,
                resolution,
            })
        })
        .collect::<Result<_>>()?;

    unsafe { CoUninitialize() };

    Ok(devices.into_iter())
}

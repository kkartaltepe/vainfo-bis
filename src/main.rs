#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Str(::std::str::Utf8Error);
        }
    }
}

use errors::*;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

fn fourcc<'a>(code: &'a u32) -> Result<&'a str> {
    let arr = unsafe { std::mem::transmute::<&u32, &[u8; 8]>(code) };
    let lower = &arr[0..4];
    return Ok(std::str::from_utf8(lower)?);
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        bail!("You must pass the DRM device");
    }
    let device = OpenOptions::new().read(true).write(true).open(&args[1])?;

    let display = unsafe { vaGetDisplayDRM(device.as_raw_fd()) };
    if display == std::ptr::null_mut() {
        bail!(format!("Failed to open display on {:?}", device))
    }

    let mut vaapi_maj = 0;
    let mut vaapi_min = 0;
    let mut status = unsafe { vaInitialize(display, &mut vaapi_maj, &mut vaapi_min) };
    if status == VA_STATUS_SUCCESS as i32 {
        println!("VA-API version {}.{} initialized", vaapi_maj, vaapi_min);
    } else {
        bail!(format!(
            "Failed to initialize VA-API on DRM display: {:?}",
            device
        ));
    }

    let mut num_profiles = unsafe { vaMaxNumProfiles(display) };
    let mut profiles: Vec<VAProfile> = Vec::with_capacity(num_profiles as usize);
    status = unsafe { vaQueryConfigProfiles(display, profiles.as_mut_ptr(), &mut num_profiles) };
    if status != VA_STATUS_SUCCESS as i32 {
        bail!(format!("vaQueryConfigProfiles failed"));
    }
    unsafe {
        profiles.set_len(num_profiles as usize);
    }
    profiles.retain(|&p| {
        let mut num_entrypoints = unsafe { vaMaxNumEntrypoints(display) };
        let mut entrypoints: Vec<VAEntrypoint> = Vec::with_capacity(num_entrypoints as usize);
        status = unsafe {
            vaQueryConfigEntrypoints(display, p, entrypoints.as_mut_ptr(), &mut num_entrypoints)
        };
        if status == VA_STATUS_ERROR_UNSUPPORTED_PROFILE as i32 {
            return false;
        }
        unsafe {
            entrypoints.set_len(num_entrypoints as usize);
        }
        entrypoints.retain(|&e| e == VAEntrypointEncSlice || e == VAEntrypointEncSliceLP);
        if entrypoints.len() > 0 {
            println!("{}:", enum_str(VAPROFILE_STR, p));
        }

        for e in entrypoints.iter() {
            let mut attrib_list = vec![
                VAConfigAttrib {
                    type_: VAConfigAttribEncMaxRefFrames,
                    value: VA_ATTRIB_NOT_SUPPORTED,
                },
                VAConfigAttrib {
                    type_: VAConfigAttribMaxPictureWidth,
                    value: VA_ATTRIB_NOT_SUPPORTED,
                },
                VAConfigAttrib {
                    type_: VAConfigAttribMaxPictureHeight,
                    value: VA_ATTRIB_NOT_SUPPORTED,
                },
                VAConfigAttrib {
                    type_: VAConfigAttribEncQualityRange,
                    value: VA_ATTRIB_NOT_SUPPORTED,
                },
                VAConfigAttrib {
                    type_: VAConfigAttribEncPackedHeaders,
                    value: VA_ATTRIB_NOT_SUPPORTED,
                },
                VAConfigAttrib {
                    type_: VAConfigAttribEncQuantization,
                    value: VA_ATTRIB_NOT_SUPPORTED,
                },
                VAConfigAttrib {
                    type_: VAConfigAttribEncMacroblockInfo,
                    value: VA_ATTRIB_NOT_SUPPORTED,
                },
            ];
            status = unsafe {
                vaGetConfigAttributes(
                    display,
                    p,
                    *e,
                    attrib_list.as_mut_ptr(),
                    attrib_list.len() as i32,
                )
            };
            if status != VA_STATUS_SUCCESS as i32 {
                return false;
            }
            let mut config = 0 as VAConfigID;
            let status =
                unsafe { vaCreateConfig(display, p, *e, std::ptr::null_mut(), 0, &mut config) };
            if status != VA_STATUS_SUCCESS as i32 || config == 0 as VAConfigID {
                return false;
            }

            let mut num_surf_attribs = 0;
            let status = unsafe {
                vaQuerySurfaceAttributes(
                    display,
                    config,
                    std::ptr::null_mut(),
                    &mut num_surf_attribs,
                )
            };
            if status != VA_STATUS_SUCCESS as i32 {
                println!("Failed to query surface attributes size");
                return false;
            }
            let mut surf_attribs =
                unsafe { vec![std::mem::zeroed::<VASurfaceAttrib>(); num_surf_attribs as usize] };
            let status = unsafe {
                vaQuerySurfaceAttributes(
                    display,
                    config,
                    surf_attribs.as_mut_ptr(),
                    &mut num_surf_attribs,
                )
            };
            if status != VA_STATUS_SUCCESS as i32 {
                println!("Failed to query surface attributes");
                return false;
            }
            println!("\t{}:", enum_str(VAENTRYPOINT_STR, *e));
            surf_attribs.retain(|attrib| return attrib.type_ != VASurfaceAttribNone);
            attrib_list.retain(|attrib| return attrib.value != VA_ATTRIB_NOT_SUPPORTED);
            for a in attrib_list.iter() {
                if a.type_ == VAConfigAttribEncMaxRefFrames {
                    println!(
                        "\t\t{}: P-Frames: {}, B-Frames: {}",
                        enum_str(VACONFIGATTRIB_STR, a.type_),
                        a.value & 0xFFFF,
                        (a.value >> 16) & 0xFFFF
                    );
                } else {
                    println!("\t\t{}:{}", enum_str(VACONFIGATTRIB_STR, a.type_), a.value);
                }
            }
            for s in surf_attribs.iter() {
                let val = unsafe { s.value.value.i };
                let name = enum_str(VASURFACEATTRIB_STR, s.type_);
                if s.type_ == VASurfaceAttribPixelFormat {
                    println!("\t\t{}:{:?}", name, fourcc(&(val as u32)));
                } else if s.type_ == VASurfaceAttribMemoryType {
                    println!("\t\t{}:0b{:b}", name, val);
                } else {
                    println!("\t\t{}:{:?}", name, val);
                }
            }
        }
        return true;
    });

    Ok(())
}

macro_rules! et {
    ($v:ident) => {
        ($v, stringify!($v))
    };
}

static VAENTRYPOINT_STR: &'static [(u32, &'static str)] = &[
    et!(VAEntrypointVLD),
    et!(VAEntrypointIZZ),
    et!(VAEntrypointIDCT),
    et!(VAEntrypointMoComp),
    et!(VAEntrypointDeblocking),
    et!(VAEntrypointEncSlice),
    et!(VAEntrypointEncPicture),
    et!(VAEntrypointEncSliceLP),
    et!(VAEntrypointVideoProc),
    et!(VAEntrypointFEI),
    et!(VAEntrypointStats),
];

static VAPROFILE_STR: &'static [(i32, &'static str)] = &[
    et!(VAProfileNone),
    et!(VAProfileMPEG2Simple),
    et!(VAProfileMPEG2Main),
    et!(VAProfileMPEG4Simple),
    et!(VAProfileMPEG4AdvancedSimple),
    et!(VAProfileMPEG4Main),
    et!(VAProfileH264Baseline),
    et!(VAProfileH264Main),
    et!(VAProfileH264High),
    et!(VAProfileVC1Simple),
    et!(VAProfileVC1Main),
    et!(VAProfileVC1Advanced),
    et!(VAProfileH263Baseline),
    et!(VAProfileJPEGBaseline),
    et!(VAProfileH264ConstrainedBaseline),
    et!(VAProfileVP8Version0_3),
    et!(VAProfileH264MultiviewHigh),
    et!(VAProfileH264StereoHigh),
    et!(VAProfileHEVCMain),
    et!(VAProfileHEVCMain10),
    et!(VAProfileVP9Profile0),
    et!(VAProfileVP9Profile1),
    et!(VAProfileVP9Profile2),
    et!(VAProfileVP9Profile3),
    et!(VAProfileHEVCMain12),
    et!(VAProfileHEVCMain422_10),
    et!(VAProfileHEVCMain422_12),
    et!(VAProfileHEVCMain444),
    et!(VAProfileHEVCMain444_10),
    et!(VAProfileHEVCMain444_12),
    et!(VAProfileHEVCSccMain),
    et!(VAProfileHEVCSccMain10),
    et!(VAProfileHEVCSccMain444),
];

static VASURFACEATTRIB_STR: &'static [(u32, &'static str)] = &[
    et!(VASurfaceAttribPixelFormat),
    et!(VASurfaceAttribMinWidth),
    et!(VASurfaceAttribMaxWidth),
    et!(VASurfaceAttribMinHeight),
    et!(VASurfaceAttribMaxHeight),
    et!(VASurfaceAttribMemoryType),
    et!(VASurfaceAttribExternalBufferDescriptor),
    et!(VASurfaceAttribUsageHint),
    et!(VASurfaceAttribCount),
];

static VACONFIGATTRIB_STR: &'static [(u32, &'static str)] = &[
    et!(VAConfigAttribRTFormat),
    et!(VAConfigAttribSpatialResidual),
    et!(VAConfigAttribSpatialClipping),
    et!(VAConfigAttribIntraResidual),
    et!(VAConfigAttribEncryption),
    et!(VAConfigAttribRateControl),
    et!(VAConfigAttribDecSliceMode),
    et!(VAConfigAttribDecJPEG),
    et!(VAConfigAttribDecProcessing),
    et!(VAConfigAttribEncPackedHeaders),
    et!(VAConfigAttribEncInterlaced),
    et!(VAConfigAttribEncMaxRefFrames),
    et!(VAConfigAttribEncMaxSlices),
    et!(VAConfigAttribEncSliceStructure),
    et!(VAConfigAttribEncMacroblockInfo),
    et!(VAConfigAttribMaxPictureWidth),
    et!(VAConfigAttribMaxPictureHeight),
    et!(VAConfigAttribEncJPEG),
    et!(VAConfigAttribEncQualityRange),
    et!(VAConfigAttribEncQuantization),
    et!(VAConfigAttribEncIntraRefresh),
    et!(VAConfigAttribEncSkipFrame),
    et!(VAConfigAttribEncROI),
    et!(VAConfigAttribEncRateControlExt),
    et!(VAConfigAttribProcessingRate),
    et!(VAConfigAttribEncDirtyRect),
    et!(VAConfigAttribEncParallelRateControl),
    et!(VAConfigAttribEncDynamicScaling),
    et!(VAConfigAttribFrameSizeToleranceSupport),
    et!(VAConfigAttribFEIFunctionType),
    et!(VAConfigAttribFEIMVPredictors),
    et!(VAConfigAttribStats),
    et!(VAConfigAttribEncTileSupport),
    et!(VAConfigAttribCustomRoundingControl),
    et!(VAConfigAttribQPBlockSize),
    et!(VAConfigAttribMaxFrameSize),
    et!(VAConfigAttribTypeMax),
];

use std::borrow::Cow;
fn enum_str<T: std::cmp::PartialEq + std::string::ToString>(
    l: &'static [(T, &'static str)],
    v: T,
) -> Cow<str> {
    return l
        .iter()
        .find(|e| e.0 == v)
        .map(|e| Cow::Borrowed(e.1))
        .unwrap_or(Cow::Owned(v.to_string()));
}

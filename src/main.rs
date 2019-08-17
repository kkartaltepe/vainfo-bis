#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            Str(::std::str::Utf8Error);
        }
    }
}

use errors::*;
use VAConfigAttribType::*;
use VAEntrypoint::*;
use VASurfaceAttribType::*;

use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

fn fourcc<'a>(code: &'a u64) -> Result<&'a str> {
    let arr = unsafe { std::mem::transmute::<&u64, &[u8; 8]>(code) };
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
    unsafe { profiles.set_len(num_profiles as usize); }
    profiles.retain(|&p| {
        let mut num_entrypoints = unsafe { vaMaxNumEntrypoints(display) };
        let mut entrypoints: Vec<VAEntrypoint> = Vec::with_capacity(num_entrypoints as usize);
        status = unsafe {
            vaQueryConfigEntrypoints(display, p, entrypoints.as_mut_ptr(), &mut num_entrypoints)
        };
        if status == VA_STATUS_ERROR_UNSUPPORTED_PROFILE as i32 {
            return false;
        }
        unsafe { entrypoints.set_len(num_entrypoints as usize); }
        entrypoints.retain(|&e| e == VAEntrypointEncSlice || e == VAEntrypointEncSliceLP);
        if entrypoints.len() > 0 {
            println!("{:?}:", p);
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
            println!("\t{:?}:", e);
            surf_attribs.retain(|attrib| return attrib.type_ != VASurfaceAttribNone);
            attrib_list.retain(|attrib| return attrib.value != VA_ATTRIB_NOT_SUPPORTED);
            for a in attrib_list.iter() {
                if a.type_ == VAConfigAttribEncMaxRefFrames {
                    println!(
                        "\t\t{:?}: P-Frames: {}, B-Frames: {}",
                        a.type_,
                        a.value & 0xFFFF,
                        (a.value >> 16) & 0xFFFF
                    );
                } else {
                    println!("\t\t{:?}:{}", a.type_, a.value);
                }
            }
            for s in surf_attribs.iter() {
                if s.type_ == VASurfaceAttribPixelFormat {
                    println!(
                        "\t\t{:?}:{:?}",
                        s.type_,
                        fourcc(&s.value.value.bindgen_union_field)
                    );
                } else if s.type_ == VASurfaceAttribMemoryType {
                    println!(
                        "\t\t{:?}:0b{:b}",
                        s.type_, s.value.value.bindgen_union_field as u32
                    );
                } else {
                    println!("\t\t{:?}:{:?}", s.type_, s.value.value.bindgen_union_field);
                }
            }
        }
        return true;
    });

    Ok(())
}

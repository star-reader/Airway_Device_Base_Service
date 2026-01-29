use crate::{AeroBase, Config};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;

#[repr(C)]
pub struct CAeroBaseConfig {
    pub db_path: *const c_char,
    pub enable_wal: bool,
    pub pool_size: u32,
}

#[repr(C)]
pub struct CDevice {
    pub id: *mut c_char,
    pub fingerprint: *mut c_char,
    pub hardware_info: *mut c_char,
    pub created_at: i64,
    pub last_seen: i64,
}

#[repr(C)]
pub struct CCoordinate {
    pub latitude: f64,
    pub longitude: f64,
}

#[repr(C)]
pub struct CAirport {
    pub id: *mut c_char,
    pub icao: *mut c_char,
    pub iata: *mut c_char,
    pub name: *mut c_char,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: i32,
    pub country: *mut c_char,
}

/// config 必须是有效的指针
#[no_mangle]
pub unsafe extern "C" fn aerobase_new(config: *const CAeroBaseConfig) -> *mut AeroBase {
    if config.is_null() {
        return ptr::null_mut();
    }

    let c_config = &*config;
    
    let db_path = if c_config.db_path.is_null() {
        PathBuf::from("aerobase.db")
    } else {
        let path_str = CStr::from_ptr(c_config.db_path).to_string_lossy();
        PathBuf::from(path_str.as_ref())
    };

    let rust_config = Config {
        db_path,
        enable_wal: c_config.enable_wal,
        pool_size: c_config.pool_size,
    };

    match tokio::runtime::Runtime::new() {
        Ok(rt) => match rt.block_on(AeroBase::new(rust_config)) {
            Ok(aerobase) => Box::into_raw(Box::new(aerobase)),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}

/// 释放 AeroBase 实例
#[no_mangle]
pub unsafe extern "C" fn aerobase_free(aerobase: *mut AeroBase) {
    if !aerobase.is_null() {
        let _ = Box::from_raw(aerobase);
    }
}

#[no_mangle]
pub unsafe extern "C" fn aerobase_get_device_fingerprint(
    aerobase: *const AeroBase,
    device: *mut CDevice,
) -> i32 {
    if aerobase.is_null() || device.is_null() {
        return -1;
    }

    let aerobase = &*aerobase;
    
    match aerobase.device().get_or_create_fingerprint() {
        Ok(dev) => {
            (*device).id = CString::new(dev.id).unwrap().into_raw();
            (*device).fingerprint = CString::new(dev.fingerprint).unwrap().into_raw();
            (*device).hardware_info = dev.hardware_info
                .map(|s| CString::new(s).unwrap().into_raw())
                .unwrap_or(ptr::null_mut());
            (*device).created_at = dev.created_at;
            (*device).last_seen = dev.last_seen;
            0
        }
        Err(_) => -1,
    }
}

/// 释放设备结构内存
#[no_mangle]
pub unsafe extern "C" fn aerobase_free_device(device: *mut CDevice) {
    if device.is_null() {
        return;
    }

    let dev = &mut *device;
    
    if !dev.id.is_null() {
        let _ = CString::from_raw(dev.id);
    }
    if !dev.fingerprint.is_null() {
        let _ = CString::from_raw(dev.fingerprint);
    }
    if !dev.hardware_info.is_null() {
        let _ = CString::from_raw(dev.hardware_info);
    }
}

#[no_mangle]
pub unsafe extern "C" fn aerobase_find_airports_within(
    aerobase: *const AeroBase,
    center: CCoordinate,
    radius_nm: f64,
    airports: *mut *mut CAirport,
    count: *mut usize,
) -> i32 {
    if aerobase.is_null() || airports.is_null() || count.is_null() {
        return -1;
    }

    let aerobase = &*aerobase;
    let coord = crate::models::Coordinate::new(center.latitude, center.longitude);

    match aerobase.spatial().find_airports_within(coord, radius_nm) {
        Ok(results) => {
            *count = results.len();
            
            if results.is_empty() {
                *airports = ptr::null_mut();
                return 0;
            }

            let mut c_airports: Vec<CAirport> = results
                .into_iter()
                .map(|ap| CAirport {
                    id: CString::new(ap.id).unwrap().into_raw(),
                    icao: CString::new(ap.icao).unwrap().into_raw(),
                    iata: ap.iata
                        .map(|s| CString::new(s).unwrap().into_raw())
                        .unwrap_or(ptr::null_mut()),
                    name: CString::new(ap.name).unwrap().into_raw(),
                    latitude: ap.coordinate.latitude,
                    longitude: ap.coordinate.longitude,
                    elevation: ap.elevation.unwrap_or(0),
                    country: ap.country
                        .map(|s| CString::new(s).unwrap().into_raw())
                        .unwrap_or(ptr::null_mut()),
                })
                .collect();

            c_airports.shrink_to_fit();
            *airports = c_airports.as_mut_ptr();
            std::mem::forget(c_airports);
            
            0
        }
        Err(_) => -1,
    }
}

/// 释放机场数组内存
#[no_mangle]
pub unsafe extern "C" fn aerobase_free_airports(airports: *mut CAirport, count: usize) {
    if airports.is_null() {
        return;
    }

    let airports_vec = Vec::from_raw_parts(airports, count, count);
    
    for airport in airports_vec {
        if !airport.id.is_null() {
            let _ = CString::from_raw(airport.id);
        }
        if !airport.icao.is_null() {
            let _ = CString::from_raw(airport.icao);
        }
        if !airport.iata.is_null() {
            let _ = CString::from_raw(airport.iata);
        }
        if !airport.name.is_null() {
            let _ = CString::from_raw(airport.name);
        }
        if !airport.country.is_null() {
            let _ = CString::from_raw(airport.country);
        }
    }
}

/// 获取最后的错误消息
#[no_mangle]
pub unsafe extern "C" fn aerobase_last_error() -> *const c_char {
    ptr::null()
}

use std::ffi::{c_void, CString};
use std::os::raw::c_char;

use reaper_medium::{
    MediaTrack, ProjectContext::CurrentProject, Reaper, ReaperString, TrackAttributeKey,
};

use crate::RouteError;

pub(crate) fn guid_to_string(guid: reaper_low::raw::GUID) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        guid.Data1,
        guid.Data2,
        guid.Data3,
        guid.Data4[0],
        guid.Data4[1],
        guid.Data4[2],
        guid.Data4[3],
        guid.Data4[4],
        guid.Data4[5],
        guid.Data4[6],
        guid.Data4[7],
    )
}

pub(crate) fn get_track_idx(reaper: &Reaper, track: MediaTrack) -> u32 {
    unsafe { reaper.get_media_track_info_value(track, TrackAttributeKey::TrackNumber) as u32 }
}

pub(crate) fn get_track_guid(reaper: &Reaper, track: MediaTrack) -> String {
    unsafe {
        let track_id = reaper.get_set_media_track_info_get_guid(track);
        guid_to_string(track_id)
    }
}

pub(crate) fn get_track_by_guid(reaper: &Reaper, guid: &str) -> Result<MediaTrack, RouteError> {
    let master_track = reaper.get_master_track(CurrentProject);
    if get_track_guid(reaper, master_track) == guid {
        return Ok(master_track);
    }
    for i in 0..reaper.count_tracks(CurrentProject) {
        let track = reaper.get_track(CurrentProject, i).unwrap();
        if get_track_guid(reaper, track) == guid {
            return Ok(track);
        }
    }
    Err(RouteError::GuidNotFound(guid.to_string()))
}

pub fn with_string_buffer<T>(
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (String, T) {
    let (cstring, result) = with_string_buffer_cstring(max_size, fill_buffer);
    (cstring.into_string().unwrap(), result)
}

pub fn with_string_buffer_cstring<T>(
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (CString, T) {
    // Using with_capacity() here wouldn't be correct because it leaves the vector length at zero.
    let vec: Vec<u8> = vec![0; max_size as usize];
    with_string_buffer_internal(vec, max_size, fill_buffer)
}

fn with_string_buffer_internal<T>(
    vec: Vec<u8>,
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (CString, T) {
    let c_string = unsafe { CString::from_vec_unchecked(vec) };
    let raw = c_string.into_raw();
    let result = fill_buffer(raw, max_size as i32);
    let cstring = unsafe { CString::from_raw(raw) };
    (cstring, result)
}

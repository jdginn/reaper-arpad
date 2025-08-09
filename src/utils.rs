use reaper_medium::{MediaTrack, ProjectContext::CurrentProject, Reaper, TrackAttributeKey};

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

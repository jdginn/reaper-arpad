use std::ffi::CStr;
use std::time::SystemTime;

use crate::{
    get_track_by_guid, get_track_guid, OscRoute, Reaper, ReceiverError, RouteError,
    TrackAttributeKey,
};
use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /num_tracks
/// - args:
///   - num_tracks (int): number of tracks in the current project
pub struct NumTracksRoute;
pub struct NumTracksParams {}
pub struct NumTracksArgs {
    pub num_tracks: i32,
}
impl OscRoute for NumTracksRoute {
    type SendParams = NumTracksArgs;
    type ReceiveParams = NumTracksParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["num_tracks"] => Some(NumTracksParams {}),
            _ => None,
        }
    }

    fn receive(_: Self::ReceiveParams, _: &OscMessage, _: &Reaper) -> Result<(), ReceiverError> {
        Ok(())
    }

    fn build_packets(args: Self::SendParams, _: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/num_tracks".to_string(),
            args: vec![OscType::Int(args.num_tracks)],
        })]
    }

    fn collect_send_params(
        _: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let num_tracks = reaper.count_tracks(reaper_medium::ProjectContext::CurrentProject) as i32;
        Ok(NumTracksArgs { num_tracks })
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/all_guids
/// - args:
///   - guids (array of string): array of unique identifiers for all tracks in the project
pub struct TrackAllGuidsRoute;
pub struct TrackAllGuidsParams {}
pub struct TrackAllGuidsArgs {
    pub guids: Vec<String>,
}
impl OscRoute for TrackAllGuidsRoute {
    type SendParams = TrackAllGuidsArgs;
    type ReceiveParams = TrackAllGuidsParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", "all_guids"] => Some(TrackAllGuidsParams {}),
            _ => None,
        }
    }

    fn receive(_: Self::ReceiveParams, _: &OscMessage, _: &Reaper) -> Result<(), ReceiverError> {
        Ok(())
    }

    fn build_packets(args: Self::SendParams, _: &Reaper) -> Vec<OscPacket> {
        let osc_args = args
            .guids
            .into_iter()
            .map(OscType::String)
            .collect::<Vec<OscType>>();
        vec![OscPacket::Message(OscMessage {
            addr: "/track/all_guids".to_string(),
            args: osc_args,
        })]
    }

    fn collect_send_params(
        _: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let mut guids = Vec::new();
        let num_tracks = reaper.count_tracks(reaper_medium::ProjectContext::CurrentProject);
        for i in 0..num_tracks {
            let track = reaper
                .get_track(reaper_medium::ProjectContext::CurrentProject, i)
                .unwrap();
            let track_guid = get_track_guid(reaper, track);
            guids.push(track_guid);
        }
        Ok(TrackAllGuidsArgs { guids })
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/index
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - index (int): index of the track in the project according to reaper's mixer view
pub struct TrackIndexRoute;
pub struct TrackIndexParams {
    track_guid: String,
}
pub struct TrackIndexArgs {
    pub track: reaper_medium::MediaTrack,
    pub index: i32,
}
impl OscRoute for TrackIndexRoute {
    type SendParams = TrackIndexArgs;
    type ReceiveParams = TrackIndexParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "index"] => Some(TrackIndexParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(_: Self::ReceiveParams, _: &OscMessage, _: &Reaper) -> Result<(), ReceiverError> {
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/index", track_guid).to_string(),
            args: vec![OscType::Int(args.index)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let index = reaper.get_media_track_info_value(track, TrackAttributeKey::TrackNumber);
            Ok(TrackIndexArgs {
                track,
                index: index as i32,
            })
        }
    }
}

/// @osc-doc
/// @writeable
/// OSC Address: /track/{track_guid}/delete
/// - params:
///   - track_guid (string): unique identifier for the track
pub struct TrackDeleteRoute;
pub struct TrackDeleteParams {
    track_guid: String,
}
pub struct TrackDeleteArgs {
    pub track_guid: String,
}
impl OscRoute for TrackDeleteRoute {
    type SendParams = TrackDeleteArgs;
    type ReceiveParams = TrackDeleteParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "delete"] => Some(TrackDeleteParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        _: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            reaper.delete_track(track);
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, _: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/delete", args.track_guid).to_string(),
            args: vec![],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        _: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        Ok(TrackDeleteArgs {
            track_guid: params.track_guid.clone(),
        })
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/name
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - name (string): name of the track
pub struct TrackNameRoute;

pub struct TrackNameParams {
    track_guid: String,
}

pub struct TrackNameArgs {
    pub track: reaper_medium::MediaTrack,
    pub name: String,
}

impl OscRoute for TrackNameRoute {
    type SendParams = TrackNameArgs;
    type ReceiveParams = TrackNameParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "name"] => Some(TrackNameParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        let name = msg.args[0].clone().string().ok_or_else(|| {
            ReceiverError::BadValue("Invalid track name, expected a string".to_string())
        })?;
        unsafe {
            reaper.get_set_media_track_info_set_name(track, name);
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/name", track_guid).to_string(),
            args: vec![OscType::String(args.name)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let name = reaper
                .get_set_media_track_info_get_name(track, |name| name.to_owned())
                .ok_or_else(|| {
                    RouteError::ValueNotFound("Failed to retrieve track name".to_string())
                })?;
            Ok(TrackNameArgs {
                track,
                name: name.to_string(),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/selected
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - selected (bool): true means track is selected
pub struct TrackSelectedRoute;

pub struct TrackSelectedParams {
    track_guid: String,
}

impl OscRoute for TrackSelectedRoute {
    type SendParams = reaper_medium::SetSurfaceSelectedArgs;
    type ReceiveParams = TrackSelectedParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "selected"] => Some(TrackSelectedParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        _: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            reaper.set_only_track_selected(Some(track));
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/selected", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_selected)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let is_selected = reaper.get_media_track_info_value(track, TrackAttributeKey::Selected);
            Ok(reaper_medium::SetSurfaceSelectedArgs {
                track,
                is_selected: (is_selected != 0.0),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/volume
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - volume (float): volume of the track, normalized to 0 to 1.0
pub struct TrackVolumeRoute;

pub struct TrackVolumeParams {
    track_guid: String,
}

impl OscRoute for TrackVolumeRoute {
    type SendParams = reaper_medium::SetSurfaceVolumeArgs;
    type ReceiveParams = TrackVolumeParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "volume"] => Some(TrackVolumeParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        let volume_raw = msg.args[0].clone().float().ok_or_else(|| {
            ReceiverError::BadValue("Invalid volume value, expected a float".to_string())
        })?;
        let slider_value = reaper_medium::VolumeSliderValue::new(
            volume_raw as f64 * reaper_medium::VolumeSliderValue::TWELVE_DB.get(),
        );
        let volume_db = reaper.slider2db(slider_value);
        let volume_linear = volume_db.to_linear_volume_value();
        unsafe {
            reaper.csurf_on_volume_change_ex(
                track,
                reaper_medium::ValueChange::Absolute(volume_linear),
                reaper_medium::GangBehavior::DenyGang,
            );
            Ok(())
        }
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        let vol_db = args.volume.to_db_ex(reaper_medium::Db::MINUS_150_DB);
        let vol_lin = reaper.db2slider(vol_db);
        let vol_norm = vol_lin.get() / reaper_medium::VolumeSliderValue::TWELVE_DB.get();
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/volume", track_guid).to_string(),
            args: vec![OscType::Float(vol_norm as f32)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let volume = reaper.get_media_track_info_value(track, TrackAttributeKey::Vol);
            Ok(reaper_medium::SetSurfaceVolumeArgs {
                track,
                volume: reaper_medium::ReaperVolumeValue::new_panic(volume),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/pan
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - pan (float): pan of the track, normalized to -1.0 to 1.0
pub struct TrackPanRoute;

pub struct TrackPanParams {
    track_guid: String,
}

impl OscRoute for TrackPanRoute {
    type SendParams = reaper_medium::SetSurfacePanArgs;
    type ReceiveParams = TrackPanParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "pan"] => Some(TrackPanParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            reaper.set_media_track_info_value(
                track,
                TrackAttributeKey::Pan,
                msg.args[0].clone().float().unwrap() as f64,
            )?;
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/pan", track_guid).to_string(),
            args: vec![OscType::Float(args.pan.into_inner() as f32)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let pan = reaper.get_media_track_info_value(track, TrackAttributeKey::Pan);
            Ok(reaper_medium::SetSurfacePanArgs {
                track,
                pan: reaper_medium::ReaperPanValue::new_panic(pan),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/mute
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - mute (bool): true means track is muted
pub struct TrackMuteRoute;

pub struct TrackMuteParams {
    track_guid: String,
}

impl OscRoute for TrackMuteRoute {
    type SendParams = reaper_medium::SetSurfaceMuteArgs;
    type ReceiveParams = TrackMuteParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "mute"] => Some(TrackMuteParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            reaper.csurf_on_mute_change_ex(
                track,
                msg.args[0].clone().bool().unwrap(),
                reaper_medium::GangBehavior::DenyGang,
            );
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/mute", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_mute)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let is_mute = reaper.get_media_track_info_value(track, TrackAttributeKey::Mute);
            Ok(reaper_medium::SetSurfaceMuteArgs {
                track,
                is_mute: (is_mute != 0.0),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/solo
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - solo (bool): true means track is soloed
pub struct TrackSoloRoute;

pub struct TrackSoloParams {
    track_guid: String,
}

impl OscRoute for TrackSoloRoute {
    type SendParams = reaper_medium::SetSurfaceSoloArgs;
    type ReceiveParams = TrackSoloParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "solo"] => Some(TrackSoloParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            reaper.csurf_on_solo_change_ex(
                track,
                msg.args[0].clone().bool().unwrap(),
                reaper_medium::GangBehavior::DenyGang,
            );
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/solo", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_solo)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let is_solo = reaper.get_media_track_info_value(track, TrackAttributeKey::Solo);
            Ok(reaper_medium::SetSurfaceSoloArgs {
                track,
                is_solo: (is_solo != 0.0),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/rec-arm
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - rec_arm (bool): true means track is armed for recording
pub struct TrackRecArmRoute;

pub struct TrackRecArmParams {
    track_guid: String,
}

impl OscRoute for TrackRecArmRoute {
    type SendParams = reaper_medium::SetSurfaceRecArmArgs;
    type ReceiveParams = TrackRecArmParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "rec-arm"] => Some(TrackRecArmParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let mode = if msg.args[0].clone().bool().unwrap() {
                reaper_medium::RecordArmMode::Armed
            } else {
                reaper_medium::RecordArmMode::Unarmed
            };
            reaper.csurf_on_rec_arm_change_ex(track, mode, reaper_medium::GangBehavior::DenyGang);
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/rec-arm", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_armed)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let is_rec_arm = reaper.get_media_track_info_value(track, TrackAttributeKey::RecArm);
            Ok(reaper_medium::SetSurfaceRecArmArgs {
                track,
                is_armed: (is_rec_arm != 0.0),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/send/{send_index}/guid
/// - params:
///   - track_guid (string): unique identifier for the track
///   - send_index (int): index of the send on the track
/// - args:
///   - guid (string): unique identifier for the send
pub struct TrackSendGuidRoute;

pub struct TrackSendGuidParams {
    track_guid: String,
    send_index: i32,
}

pub struct TrackSendGuidArgs {
    pub track: reaper_medium::MediaTrack,
    pub send_index: i32,
    pub send_guid: String,
}

impl OscRoute for TrackSendGuidRoute {
    type SendParams = TrackSendGuidArgs;
    type ReceiveParams = TrackSendGuidParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "send", send_index, "guid"] => Some(TrackSendGuidParams {
                track_guid: track_guid.to_string(),
                send_index: send_index.parse().ok()?,
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        _: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let _ = get_track_by_guid(reaper, &params.track_guid)?;
        // This route is read-only, so we don't need to do anything here.
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/send/{}/guid", track_guid, args.send_index).to_string(),
            args: vec![OscType::String(args.send_guid)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let send_track = reaper
                .get_track_send_info_desttrack(
                    track,
                    reaper_medium::TrackSendDirection::Send,
                    params.send_index as u32,
                )
                .map_err(|_| {
                    RouteError::ValueNotFound("Failed to retrieve send track".to_string())
                })?;
            let send_guid = get_track_guid(reaper, send_track);
            Ok(TrackSendGuidArgs {
                track,
                send_index: params.send_index,
                send_guid,
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/send/{send_index}/volume
/// - params:
///   - track_guid (string): unique identifier for the track
///   - send_index (int): index of the send on the track
/// - args:
///   - volume (float): volume of the send, normalized to 0 to 1.
pub struct TrackSendVolumeRoute;

pub struct TrackSendVolumeParams {
    track_guid: String,
    send_index: i32,
}

impl OscRoute for TrackSendVolumeRoute {
    type SendParams = reaper_medium::ExtSetSendVolumeArgs;
    type ReceiveParams = TrackSendVolumeParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "send", send_index, "volume"] => Some(TrackSendVolumeParams {
                track_guid: track_guid.to_string(),
                send_index: send_index.parse().ok()?,
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let track_send_ref = reaper_medium::TrackSendRef::Send(
                u32::try_from(params.send_index)
                    .map_err(|_| ReceiverError::BadValue("Invalid send index".to_string()))?,
            );
            let volume =
                reaper_medium::ReaperVolumeValue::new(msg.args[0].clone().float().unwrap() as f64)
                    .map_err(|_| ReceiverError::BadValue("Invalid volume value".to_string()))?;
            reaper.set_track_send_ui_vol(
                track,
                track_send_ref,
                volume,
                reaper_medium::EditMode::NormalTweak,
            )?
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/send/{}/volume", track_guid, args.send_index).to_string(),
            args: vec![OscType::Float(args.volume.into_inner() as f32)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let volume = reaper.get_track_send_info_value(
                track,
                reaper_medium::TrackSendCategory::Send,
                params.send_index as u32,
                reaper_medium::TrackSendAttributeKey::Vol,
            );
            Ok(reaper_medium::ExtSetSendVolumeArgs {
                track,
                send_index: params.send_index as u32,
                volume: reaper_medium::ReaperVolumeValue::new_panic(volume),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/send/{send_index}/pan
/// - params:
///   - track_guid (string): unique identifier for the track
///   - send_index (int): index of the send on the track
/// - args:
///   - pan (float): pan of the send, normalized to -1.0 to 1.0
pub struct TrackSendPanRoute;

pub struct TrackSendPanParams {
    track_guid: String,
    send_index: i32,
}

impl OscRoute for TrackSendPanRoute {
    type SendParams = reaper_medium::ExtSetSendPanArgs;
    type ReceiveParams = TrackSendPanParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "send", send_index, "pan"] => Some(TrackSendPanParams {
                track_guid: track_guid.to_string(),
                send_index: send_index.parse().ok()?,
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let track_send_ref = reaper_medium::TrackSendRef::Send(
                u32::try_from(params.send_index)
                    .map_err(|_| ReceiverError::BadValue("Invalid send index".to_string()))?,
            );
            let pan =
                reaper_medium::ReaperPanValue::new(msg.args[0].clone().float().unwrap() as f64)
                    .map_err(|_| ReceiverError::BadValue("Invalid pan value".to_string()))?;
            reaper.set_track_send_ui_pan(
                track,
                track_send_ref,
                pan,
                reaper_medium::EditMode::NormalTweak,
            )?
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/send/{}/pan", track_guid, args.send_index).to_string(),
            args: vec![OscType::Float(args.pan.into_inner() as f32)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let pan = reaper.get_track_send_info_value(
                track,
                reaper_medium::TrackSendCategory::Send,
                params.send_index as u32,
                reaper_medium::TrackSendAttributeKey::Pan,
            );
            Ok(reaper_medium::ExtSetSendPanArgs {
                track,
                send_index: params.send_index as u32,
                pan: reaper_medium::ReaperPanValue::new_panic(pan),
            })
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/color
/// - params:
///   - track_guid (string): unique identifier for the track
/// - args:
///   - color (int): color of the track, represented as an RGB integer
pub struct TrackColorRoute;
pub struct TrackColorParams {
    track_guid: String,
}
pub struct TrackColorArgs {
    pub track: reaper_medium::MediaTrack,
    pub color: i32,
}

impl OscRoute for TrackColorRoute {
    type SendParams = TrackColorArgs;
    type ReceiveParams = TrackColorParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "color"] => Some(TrackColorParams {
                track_guid: track_guid.to_string(),
            }),
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let int_arg = msg.args[0].clone().int().ok_or_else(|| {
                ReceiverError::BadValue("Invalid color value, expected an integer".to_string())
            })?;
            reaper.get_set_media_track_info_set_custom_color(
                track,
                reaper_medium::NativeColorValue {
                    color: reaper_medium::NativeColor::new(int_arg),
                    is_used: true,
                },
            );
        }
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        let track_guid = get_track_guid(reaper, args.track);
        vec![OscPacket::Message(OscMessage {
            addr: format!("/track/{}/color", track_guid).to_string(),
            args: vec![OscType::Int(args.color)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let color = reaper.get_set_media_track_info_get_custom_color(track);
            Ok(TrackColorArgs {
                track,
                color: color.color.to_raw(),
            })
        }
    }
}

// TODO: what routes do we need?
//
// Need a way to get all FX on a track
//  - Maybe we just always send everything and let the other side cache what they want to keep?
// Need a way to get all params for some FX
// Need a way to receive parameter changes for FX (avoid being too noisy about it)

/// @osc-doc
/// @readonly
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/info
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - fx_idx (int): index of the FX on the track
/// Replies with many OSC messages reporting the folowing:
/// - name (string): name of the FX
/// - param_count (int): number of parameters for the FX
/// - for each param:
///   - param_idx (int): index of the parameter
///   - param_name (string): name of the parameter
///   - param_value (float): current value of the parameter, normalized to 0.
///   - param_min (float): minimum value of the parameter, normalized to 0. TODO: what here?
///   - param_max (float): maximum value of the parameter, normalized to 1.0
///   - param_default (float): default value of the parameter, normalized to 0.
pub struct TrackFXInfoRoute;
pub struct TrackFXInfoParams {
    track_guid: String,
    fx_idx: u32,
}
#[derive(Clone)]
pub struct ParamInfo {
    pub param_name: String,
    pub param_value: f64,
    pub param_min: f64,
    pub param_max: f64,
    // pub param_step_size: Option<f64>,
}
#[derive(Clone)]
pub struct FxInfo {
    pub name: String,
    pub ident: String,
    pub param_count: u32,
    pub params: Vec<ParamInfo>,
}
pub struct TrackFXInfoArgs {
    pub track: reaper_medium::MediaTrack,
    pub fx_info: FxInfo,
}

impl OscRoute for TrackFXInfoRoute {
    type SendParams = TrackFXInfoArgs;
    type ReceiveParams = TrackFXInfoParams;

    fn matcher(_segments: &[&str]) -> Option<Self::ReceiveParams> {
        None
    }

    fn receive(
        _params: Self::ReceiveParams,
        _msg: &OscMessage,
        _reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        Ok(())
    }

    fn build_packets(_args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "".to_string(),
            args: vec![],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let fx_name = reaper
                .track_fx_get_fx_name(
                    track,
                    reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
                    24, // Name length in bytes TODO: what size makes sense here?
                )
                .unwrap();
            Ok(TrackFXInfoArgs {
                track,
                fx_info: FxInfo {
                    name: fx_name.to_string(),
                    ident: "".to_string(),
                    param_count: reaper.track_fx_get_num_params(
                        track,
                        reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
                    ),
                    params: vec![],
                },
            })
        }
    }
}

pub struct TrackFXParamRoute;
pub struct TrackFXParamParams {
    track_guid: String,
    fx_idx: u32,
    param_idx: u32,
}
pub struct TrackFXParamArgs {
    // pub track: reaper_medium::MediaTrack,
    // pub param_value: f64,
}
impl OscRoute for TrackFXParamRoute {
    type SendParams = TrackFXParamArgs;
    type ReceiveParams = TrackFXParamParams;

    fn matcher(_segments: &[&str]) -> Option<Self::ReceiveParams> {
        None
    }

    fn receive(
        _params: Self::ReceiveParams,
        _msg: &OscMessage,
        _reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        Ok(())
    }

    fn build_packets(_args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "".to_string(),
            args: vec![],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            let fx_location = reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx);
            let num_params = reaper.track_fx_get_num_params(
                track,
                reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
            );
            for ident in 0..num_params {
                let param_name = reaper.track_fx_get_param_name(
                    track,
                    fx_location,
                    params.param_idx,
                    128, // Name length in bytes TODO: what size makes sense here?
                );
                let res = reaper.track_fx_get_param_ex(track, fx_location, params.param_idx);
            }
            Ok(TrackFXParamArgs {})
        }
    }
}

/// @osc-doc
/// @readonly
/// OSC Address: /fxinfo
/// Arguments:
/// - none
/// Replies with many OSC messages reporting the following for all FX on all tracks:
///
/// @osc-doc
/// @readonly
/// OSC Address: /fxinfo/{ident}/name
/// Arguments:
/// - ident (string): unique identifier for the FX
/// - name (string): name of the FX
///
/// @osc-doc
/// @readonly
/// OSC Address: /fxinfo/{ident}/param_count
/// Arguments:
/// - ident (string): unique identifier for the FX
/// - param_count (int): number of parameters for the FX
///
/// @osc-doc
/// @readonly
/// OSC Address: /fxinfo/{ident}/param/{param_idx}/name
/// Arguments:
/// - ident (string): unique identifier for the FX
/// - param_idx (int): index of the parameter
/// - param_name (string): name of the parameter
///
/// @osc-doc
/// @readonly
/// OSC Address: /fxinfo/{ident}/param/{param_idx}/value
/// Arguments:
/// - ident (string): unique identifier for the FX
/// - param_idx (int): index of the parameter
/// - param_value (float): current raw value of the parameter
///
/// @osc-doc
/// @readonly
/// OSC Address: /fxinfo/{ident}/param/{param_idx}/min
/// Arguments:
/// - ident (string): unique identifier for the FX
/// - param_idx (int): index of the parameter
/// - param_min (float): minimum raw value of the parameter
///
/// @osc-doc
/// @readonly
/// OSC Address: /fxinfo/{ident}/param/{param_idx}/max
/// Arguments:
/// - ident (string): unique identifier for the FX
/// - param_idx (int): index of the parameter
/// - param_max (float): maximum raw value of the parameter
pub struct AllFxInfoRoute;
pub struct AllFxInfoParams;
pub struct AllFxInfoArgs {
    pub fx: Vec<FxInfo>,
}

impl OscRoute for AllFxInfoRoute {
    type SendParams = AllFxInfoArgs;
    type ReceiveParams = AllFxInfoParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["fxinfo"] => Some(AllFxInfoParams {}),
            _ => None,
        }
    }

    fn receive(
        _params: Self::ReceiveParams,
        _msg: &OscMessage,
        _reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        Ok(())
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        args.fx
            .iter()
            .flat_map(|fx_info| {
                let mut messages = vec![];
                let ident = format!("<{}>", fx_info.ident.clone());
                // messages.push(OscPacket::Message(OscMessage {
                //     addr: format!("/fxinfo/{}/ident", ident).to_string(),
                //     args: vec![OscType::String(fx_info.ident.clone())],
                // }));
                messages.push(OscPacket::Message(OscMessage {
                    addr: format!("/fxinfo/{}/name", ident).to_string(),
                    args: vec![OscType::String(fx_info.name.clone())],
                }));
                messages.push(OscPacket::Message(OscMessage {
                    addr: format!("/fxinfo/{}/param_count", ident).to_string(),
                    args: vec![OscType::Int(fx_info.param_count.clone() as i32)],
                }));
                for (param_idx, param) in fx_info.params.clone().into_iter().enumerate() {
                    messages.push(OscPacket::Message(OscMessage {
                        addr: format!("/fxinfo/{}/param/{}/name", ident, param_idx),
                        args: vec![OscType::String(param.param_name.clone())],
                    }));
                    // messages.push(OscPacket::Message(OscMessage {
                    //     addr: format!("/fxinfo/{}/param/{}/value", ident, param_idx),
                    //     args: vec![OscType::Float(param.param_value as f32)],
                    // }));
                    messages.push(OscPacket::Message(OscMessage {
                        addr: format!("/fxinfo/{}/param/{}/min", ident, param_idx),
                        args: vec![OscType::Float(param.param_min as f32)],
                    }));
                    messages.push(OscPacket::Message(OscMessage {
                        addr: format!("/fxinfo/{}/param/{}/max", ident, param_idx),
                        args: vec![OscType::Float(param.param_max as f32)],
                    }));
                    // messages.push(OscPacket::Message(OscMessage {
                    //     addr: format!("/fxinfo/{}/param/{}/default", ident, param_idx),
                    //     args: vec![OscType::Float(param.param_default as f32)],
                    // }));
                }
                // println!("This FX messages: {:?}", messages);
                messages
            })
            .collect()
        // vec![OscPacket::Bundle(OscBundle {
        //     timetag: OscTime::try_from(now).unwrap(),
        //     content: args
        //         .fx
        //         .into_iter()
        //         .flat_map(|fx_info| {
        //             let mut messages = vec![];
        //             let ident = fx_info.name.clone();
        //             // messages.push(OscPacket::Message(OscMessage {
        //             //     addr: format!("/fxinfo/{}/ident", ident).to_string(),
        //             //     args: vec![OscType::String(fx_info.ident.clone())],
        //             // }));
        //             messages.push(OscPacket::Message(OscMessage {
        //                 addr: format!("/fxinfo/{}/name", ident).to_string(),
        //                 args: vec![OscType::String(fx_info.name.clone())],
        //             }));
        //             messages.push(OscPacket::Message(OscMessage {
        //                 addr: format!("/fxinfo/{}/param_count", ident).to_string(),
        //                 args: vec![OscType::Int(fx_info.param_count as i32)],
        //             }));
        //             for (param_idx, param) in fx_info.params.into_iter().enumerate() {
        //                 messages.push(OscPacket::Message(OscMessage {
        //                     addr: format!("/fxinfo/{}/param/{}/name", ident, param_idx),
        //                     args: vec![OscType::String(param.param_name.clone())],
        //                 }));
        //                 // messages.push(OscPacket::Message(OscMessage {
        //                 //     addr: format!("/fxinfo/{}/param/{}/value", ident, param_idx),
        //                 //     args: vec![OscType::Float(param.param_value as f32)],
        //                 // }));
        //                 messages.push(OscPacket::Message(OscMessage {
        //                     addr: format!("/fxinfo/{}/param/{}/min", ident, param_idx),
        //                     args: vec![OscType::Float(param.param_min as f32)],
        //                 }));
        //                 messages.push(OscPacket::Message(OscMessage {
        //                     addr: format!("/fxinfo/{}/param/{}/max", ident, param_idx),
        //                     args: vec![OscType::Float(param.param_max as f32)],
        //                 }));
        //                 // messages.push(OscPacket::Message(OscMessage {
        //                 //     addr: format!("/fxinfo/{}/param/{}/default", ident, param_idx),
        //                 //     args: vec![OscType::Float(param.param_default as f32)],
        //                 // }));
        //             }
        //             // println!("This FX messages: {:?}", messages);
        //             messages
        //         })
        //         .collect(),
        // })]
    }

    fn collect_send_params(
        _params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        fn enum_installed_fx(reaper: &Reaper, index: i32) -> Option<(String, String)> {
            // Allocate zero-initialized buffers
            unsafe {
                let name_buf = std::ffi::CString::from_vec_unchecked(vec![0u8; 256]);
                let ident_buf = std::ffi::CString::from_vec_unchecked(vec![0u8; 256]);

                // Prepare mutable raw pointers to store `*const i8`
                let mut name_ptr: *const i8 = name_buf.as_ptr();
                let mut ident_ptr: *const i8 = ident_buf.as_ptr();

                // Cast mutable references to mutable pointers
                let name_out_ptr: *mut *const i8 = &mut name_ptr;
                let ident_out_ptr: *mut *const i8 = &mut ident_ptr;

                // Make the unsafe FFI call
                if !reaper
                    .low()
                    .EnumInstalledFX(index, name_out_ptr, ident_out_ptr)
                {
                    return None;
                }

                // Convert results from raw buffer to Strings
                let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();

                let ident = CStr::from_ptr(ident_ptr).to_string_lossy().into_owned();

                // println!("Converted Name: {}, Ident: {}", name, ident);
                Some((name, ident))
            }
        }

        let mut fx_infos = vec![];

        // Insert a temporary track to enumerate FX
        reaper.insert_track_at_index(0, reaper_medium::TrackDefaultsBehavior::OmitDefaultEnvAndFx);
        let track = reaper
            .get_track(reaper_medium::ProjectContext::CurrentProject, 0)
            .ok_or_else(|| {
                RouteError::ValueNotFound(
                    "Failed to create temporary track for FX enumeration".to_string(),
                )
            })?;

        let mut i = 0;
        while let Some((name, ident)) = enum_installed_fx(reaper, i) {
            let mut params = vec![];
            unsafe {
                // println!("Adding FX {} ({}) to temporary track", name, ident);
                let fx_index = reaper
                    .track_fx_add_by_name_add(
                        track,
                        name.clone(),
                        reaper_medium::TrackFxChainType::NormalFxChain,
                        reaper_medium::AddFxBehavior::AddIfNotFound,
                    )
                    .unwrap();
                // println!("Added FX {} at index {}", name, fx_index);
                let num_params = reaper.track_fx_get_num_params(
                    track,
                    reaper_medium::TrackFxLocation::NormalFxChain(fx_index),
                );
                for param_idx in 0..num_params {
                    let param_name = reaper
                        .track_fx_get_param_name(
                            track,
                            reaper_medium::TrackFxLocation::NormalFxChain(fx_index),
                            param_idx,
                            128, // Name length in bytes TODO: what size makes sense here?
                        )
                        .unwrap_or_default();
                    let param_ex = reaper.track_fx_get_param_ex(
                        track,
                        reaper_medium::TrackFxLocation::NormalFxChain(fx_index),
                        param_idx,
                    );
                    // let param_step_size = reaper.track_fx_get_parameter_step_sizes(
                    //     track,
                    //     reaper_medium::TrackFxLocation::NormalFxChain(fx_index),
                    //     param_idx,
                    // );
                    // println!(
                    //     "    Param {:?}: {:?} = {:?} / {:?} / {:?}",
                    //     param_idx,
                    //     param_name,
                    //     param_ex.min_value,
                    //     param_ex.current_value,
                    //     param_ex.max_value
                    // );
                    params.push(ParamInfo {
                        param_name: "foo".to_string(), //TODO: fixme
                        param_value: param_ex.current_value,
                        param_min: param_ex.min_value,
                        param_max: param_ex.max_value,
                    });
                }
                // println!("FX {}: {} ({})", i, name, ident);
                fx_infos.push(FxInfo {
                    name,
                    ident,
                    param_count: num_params,
                    params: vec![],
                });
            }
            i += 1;
        }
        unsafe { reaper.delete_track(track) };
        Ok(AllFxInfoArgs { fx: fx_infos })
    }
}

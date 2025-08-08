use crate::{
    get_track_by_guid, get_track_guid, OscRoute, Reaper, ReceiverError, RouteError,
    TrackAttributeKey,
};
use rosc::{OscMessage, OscType};

/// @osc-doc
/// @readonly
/// OSC Address: /track/{track_guid}/index
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - index (int): index of the track in the project according to reaper's mixer view
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/index", track_guid).to_string(),
            args: vec![OscType::Int(args.index)],
        }
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
/// OSC Address: /track/{track_guid}/name
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - name (string): name of the track
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/name", track_guid).to_string(),
            args: vec![OscType::String(args.name)],
        }
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
/// OSC Address: /track/{track_guid}/selected
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - selected (bool): true means track is selected
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
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.track_guid)?;
        unsafe {
            reaper.set_media_track_info_value(
                track,
                TrackAttributeKey::Selected,
                msg.args[0].clone().int().unwrap() as f64,
            )?;
        }
        Ok(())
    }

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/selected", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_selected)],
        }
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
/// OSC Address: /track/{track_guid}/volume
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - volume (float): volume of the track, normalized to 0 to 1.0
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        let vol_db = args.volume.to_db_ex(reaper_medium::Db::MINUS_150_DB);
        let vol_lin = reaper.db2slider(vol_db);
        let vol_norm = vol_lin.get() / reaper_medium::VolumeSliderValue::TWELVE_DB.get();
        OscMessage {
            addr: format!("/track/{}/volume", track_guid).to_string(),
            args: vec![OscType::Float(vol_norm as f32)],
        }
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
/// OSC Address: /track/{track_guid}/pan
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - pan (float): pan of the track, normalized to -1.0 to 1.0
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/pan", track_guid).to_string(),
            args: vec![OscType::Float(args.pan.into_inner() as f32)],
        }
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
/// OSC Address: /track/{track_guid}/mute
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - mute (bool): true means track is muted
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
            reaper.set_media_track_info_value(
                track,
                TrackAttributeKey::Mute,
                msg.args[0].clone().int().unwrap() as f64,
            )?;
        }
        Ok(())
    }

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/mute", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_mute)],
        }
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
/// OSC Address: /track/{track_guid}/solo
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - solo (bool): true means track is soloed
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
            reaper.set_media_track_info_value(
                track,
                TrackAttributeKey::Solo,
                msg.args[0].clone().int().unwrap() as f64,
            )?;
        }
        Ok(())
    }

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/solo", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_solo)],
        }
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
/// OSC Address: /track/{track_guid}/rec-arm
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - rec_arm (bool): true means track is armed for recording
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
            reaper.set_media_track_info_value(
                track,
                TrackAttributeKey::RecArm,
                msg.args[0].clone().int().unwrap() as f64,
            )?;
        }
        Ok(())
    }

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/rec-arm", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_armed)],
        }
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
/// @readonly
/// OSC Address: /track/{track_guid}/send/{send_index}/guid
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - send_index (int): index of the send on the track
/// - guid (string): unique identifier for the send
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/send/{}/guid", track_guid, args.send_index).to_string(),
            args: vec![OscType::String(args.send_guid)],
        }
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
/// OSC Address: /track/{track_guid}/send/{send_index}/volume
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - send_index (int): index of the send on the track
/// - volume (float): volume of the send, normalized to 0 to 1.
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/send/{}/volume", track_guid, args.send_index).to_string(),
            args: vec![OscType::Float(args.volume.into_inner() as f32)],
        }
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
/// OSC Address: /track/{track_guid}/send/{send_index}/pan
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - send_index (int): index of the send on the track
/// - pan (float): pan of the send, normalized to -1.0 to 1.0
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/send/{}/pan", track_guid, args.send_index).to_string(),
            args: vec![OscType::Float(args.pan.into_inner() as f32)],
        }
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
/// OSC Address: /track/{track_guid}/color
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - color (int): color of the track, represented as an RGB integer
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

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        OscMessage {
            addr: format!("/track/{}/color", track_guid).to_string(),
            args: vec![OscType::Int(args.color)],
        }
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

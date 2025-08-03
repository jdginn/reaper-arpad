use crate::{
    get_track_by_guid, get_track_guid, OscRoute, Reaper, ReceiverError, RouteError,
    TrackAttributeKey,
};
use reaper_medium;
use rosc::{OscMessage, OscType};

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
    track: reaper_medium::MediaTrack,
    name: String,
}

impl OscRoute for TrackNameRoute {
    type SendParams = TrackNameArgs;
    type ReceiveParams = TrackNameParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "name"] => Some(TrackNameParams {
                track_guid: track_guid.to_string(),
            }),
            _ => {
                return None;
            }
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
        unsafe {
            reaper.get_set_media_track_info_set_name(track, msg.args[0].clone().string().unwrap());
        }
        Ok(())
    }

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        return OscMessage {
            addr: format!("/track/{}/name", track_guid).to_string(),
            args: vec![OscType::String(args.name)],
        };
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
            _ => {
                return None;
            }
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
        return OscMessage {
            addr: format!("/track/{}/selected", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_selected)],
        };
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
            _ => {
                return None;
            }
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
        unsafe {
            reaper.set_media_track_info_value(
                track,
                TrackAttributeKey::Vol,
                msg.args[0].clone().float().unwrap() as f64,
            )?;
            Ok(())
        }
    }

    fn build_message(args: Self::SendParams, reaper: &Reaper) -> OscMessage {
        let track_guid = get_track_guid(reaper, args.track);
        return OscMessage {
            addr: format!("/track/{}/volume", track_guid).to_string(),
            args: vec![OscType::Float(args.volume.into_inner() as f32)],
        };
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
        unsafe {
            let volume = reaper.get_media_track_info_value(track, TrackAttributeKey::Vol);
            return Ok(reaper_medium::SetSurfaceVolumeArgs {
                track,
                volume: reaper_medium::ReaperVolumeValue::new_panic(volume),
            });
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
            _ => {
                return None;
            }
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
        return OscMessage {
            addr: format!("/track/{}/pan", track_guid).to_string(),
            args: vec![OscType::Float(args.pan.into_inner() as f32)],
        };
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
            _ => {
                return None;
            }
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
        return OscMessage {
            addr: format!("/track/{}/mute", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_mute)],
        };
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
            _ => {
                return None;
            }
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
        return OscMessage {
            addr: format!("/track/{}/solo", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_solo)],
        };
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
            _ => {
                return None;
            }
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
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
        return OscMessage {
            addr: format!("/track/{}/rec-arm", track_guid).to_string(),
            args: vec![OscType::Bool(args.is_armed)],
        };
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(&reaper, &params.track_guid)?;
        unsafe {
            let is_rec_arm = reaper.get_media_track_info_value(track, TrackAttributeKey::RecArm);
            Ok(reaper_medium::SetSurfaceRecArmArgs {
                track,
                is_armed: (is_rec_arm != 0.0),
            })
        }
    }
}

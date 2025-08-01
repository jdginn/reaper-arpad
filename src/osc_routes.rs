use crate::{
    get_track_by_guid, get_track_guid, OscRoute, Reaper, ReceiverError, RouteError,
    TrackAttributeKey,
};
use reaper_medium;
use rosc::{OscMessage, OscType};

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
/// - mute (int): 0 to mute, 1 to unmute
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

use std::ffi::{CStr, CString};
use std::time::SystemTime;

use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};

use crate::{get_track_by_guid, OscRoute, Reaper, ReceiverError, RouteError};

fn fx_guid_to_string(guid: reaper_low::raw::GUID) -> String {
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
        guid.Data4[7]
    )
}

#[derive(Clone)]
pub struct ParamInfo {
    pub param_name: String,
    pub param_value: f64,
    pub param_min: f64,
    pub param_max: f64,
    pub param_step_size: Option<f64>,
}
#[derive(Clone)]
pub struct FxInfo {
    pub name: String,
    pub guid: reaper_low::raw::GUID,
    pub enabled: bool,
    pub num_params: u32,
    pub params: Vec<ParamInfo>,
}

pub struct TrackFXParamParams {
    track_guid: String,
    fx_idx: u32,
    param_idx: u32,
}
pub struct TrackFXParamArgs {
    track_guid: String,
    fx_idx: u32,
    param_idx: u32,
    param_info: ParamInfo,
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/guid
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
/// - args:
///   - guid (string): unique identifier for the FX
// TODO: make this writeable to allow changing the FX in this slot.
// TODO: add a route to set FX by name.
pub struct TrackFXGuidRoute;

impl OscRoute for TrackFXGuidRoute {
    type SendParams = String;
    type ReceiveParams = (String, u32); // (track_guid, fx_idx)

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "guid"] => {
                Some((track_guid.to_string(), fx_idx.parse().ok()?))
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/track/{track_guid}/fx/{fx_idx}/guid".to_string(),
            args: vec![OscType::String(args)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.0)?;
        unsafe {
            let fx_guid = reaper
                .track_fx_get_fx_guid(
                    track,
                    reaper_medium::TrackFxLocation::NormalFxChain(params.1),
                )
                .unwrap();
            Ok(fx_guid_to_string(fx_guid))
        }
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/name
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
/// - args:
///   - name (string): name of the FX
pub struct TrackFXNameRoute;

impl OscRoute for TrackFXNameRoute {
    type SendParams = String;
    type ReceiveParams = (String, u32); // (track_guid, fx_idx)

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "name"] => {
                Some((track_guid.to_string(), fx_idx.parse().ok()?))
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/track/{track_guid}/fx/{fx_idx}/name".to_string(),
            args: vec![OscType::String(args)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.0)?;
        unsafe {
            let fx_name = reaper
                .track_fx_get_fx_name(
                    track,
                    reaper_medium::TrackFxLocation::NormalFxChain(params.1),
                    24, // Name length in bytes TODO: what size makes sense here?
                )
                .unwrap();
            Ok(fx_name.to_string())
        }
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/enabled
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
/// - args:
///   - enabled (bool): true if the FX is enabled
pub struct TrackFXEnabledRoute;
impl OscRoute for TrackFXEnabledRoute {
    type SendParams = bool;
    type ReceiveParams = (String, u32); // (track_guid, fx_idx)

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "enabled"] => {
                Some((track_guid.to_string(), fx_idx.parse().ok()?))
            }
            _ => None,
        }
    }

    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        let track = get_track_by_guid(reaper, &params.0)?;
        let enabled = msg.args[0].clone().bool().unwrap();
        unsafe {
            Ok(reaper.track_fx_set_enabled(
                track,
                reaper_medium::TrackFxLocation::NormalFxChain(params.1),
                enabled,
            ))
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/track/{track_guid}/fx/{fx_idx}/enabled".to_string(),
            args: vec![OscType::Bool(args)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.0)?;
        unsafe {
            Ok(reaper.track_fx_get_enabled(
                track,
                reaper_medium::TrackFxLocation::NormalFxChain(params.1),
            ))
        }
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param_count
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
/// - args:
///   - param_count (int): number of parameters for the FX
pub struct TrackFXParamCountRoute;

impl OscRoute for TrackFXParamCountRoute {
    type SendParams = u32;
    type ReceiveParams = (String, u32); // (track_guid, fx_idx)

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "param_count"] => {
                Some((track_guid.to_string(), fx_idx.parse().ok()?))
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/track/{track_guid}/fx/{fx_idx}/param_count".to_string(),
            args: vec![OscType::Int(args as i32)],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        let track = get_track_by_guid(reaper, &params.0)?;
        unsafe {
            let num_params = reaper.track_fx_get_num_params(
                track,
                reaper_medium::TrackFxLocation::NormalFxChain(params.1),
            );
            Ok(num_params)
        }
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param/{param_idx}/name
/// - params:
///  - track_guid (string): unique identifier for the track
///  - fx_idx (int): index of the FX on the track
///  - param_idx (int): index of the parameter
///  - args:
///  - param_name (string): name of the parameter
pub struct TrackFXParamNameRoute;

impl OscRoute for TrackFXParamNameRoute {
    type SendParams = TrackFXParamArgs;
    type ReceiveParams = TrackFXParamParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "param", param_idx, "name"] => {
                Some(TrackFXParamParams {
                    track_guid: track_guid.to_string(),
                    fx_idx: fx_idx.parse().ok()?,
                    param_idx: param_idx.parse().ok()?,
                })
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: format!(
                "/track/{}/fx/{}/param/{}/name",
                args.track_guid, args.fx_idx, args.param_idx
            ),
            args: vec![OscType::String(args.param_info.param_name.clone())],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        collect_track_param_info(params, reaper).unwrap();
        Ok(TrackFXParamArgs {
            track_guid: params.track_guid.clone(),
            fx_idx: params.fx_idx,
            param_idx: params.param_idx,
            param_info: collect_track_param_info(params, reaper)?,
        })
    }
}

/// @osc-doc
/// @readable
/// @writeable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param/{param_idx}/value
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
///   - param_idx (int): index of the parameter
/// - args:
///   - value (float): value of the parameter
pub struct TrackFXParamValueRoute;
impl OscRoute for TrackFXParamValueRoute {
    type SendParams = TrackFXParamArgs;
    type ReceiveParams = TrackFXParamParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "param", param_idx, "value"] => {
                Some(TrackFXParamParams {
                    track_guid: track_guid.to_string(),
                    fx_idx: fx_idx.parse().ok()?,
                    param_idx: param_idx.parse().ok()?,
                })
            }
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
            reaper
                .track_fx_set_param_normalized(
                    track,
                    reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
                    params.param_idx,
                    reaper_medium::ReaperNormalizedFxParamValue::from(
                        msg.args[0].clone().float().unwrap() as f64,
                    ),
                )
                .map_err(ReceiverError::Reaper)
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Bundle(OscBundle {
            timetag: OscTime::try_from(SystemTime::now()).unwrap(),
            content: vec![OscPacket::Message(OscMessage {
                addr: format!(
                    "/track/{}/fx/{}/param/{}/value",
                    args.track_guid, args.fx_idx, args.param_idx
                ),
                args: vec![OscType::Float(args.param_info.param_value as f32)],
            })],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        collect_track_param_info(params, reaper).unwrap();
        Ok(TrackFXParamArgs {
            track_guid: params.track_guid.clone(),
            fx_idx: params.fx_idx,
            param_idx: params.param_idx,
            param_info: collect_track_param_info(params, reaper)?,
        })
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param/{param_idx}/min
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
///   - param_idx (int): index of the parameter
/// - args:
///   - min (float): minimum value of the parameter
pub struct TrackFXParamMinRoute;
impl OscRoute for TrackFXParamMinRoute {
    type SendParams = TrackFXParamArgs;
    type ReceiveParams = TrackFXParamParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "param", param_idx, "min"] => {
                Some(TrackFXParamParams {
                    track_guid: track_guid.to_string(),
                    fx_idx: fx_idx.parse().ok()?,
                    param_idx: param_idx.parse().ok()?,
                })
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Bundle(OscBundle {
            timetag: OscTime::try_from(SystemTime::now()).unwrap(),
            content: vec![OscPacket::Message(OscMessage {
                addr: format!(
                    "/track/{}/fx/{}/param/{}/min",
                    args.track_guid, args.fx_idx, args.param_idx
                ),
                args: vec![OscType::Float(args.param_info.param_min as f32)],
            })],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        collect_track_param_info(params, reaper).unwrap();
        Ok(TrackFXParamArgs {
            track_guid: params.track_guid.clone(),
            fx_idx: params.fx_idx,
            param_idx: params.param_idx,
            param_info: collect_track_param_info(params, reaper)?,
        })
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param/{param_idx}/max
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
///   - param_idx (int): index of the parameter
/// - args:
///   - max (float): maximum value of the parameter
pub struct TrackFXParamMaxRoute;
impl OscRoute for TrackFXParamMaxRoute {
    type SendParams = TrackFXParamArgs;
    type ReceiveParams = TrackFXParamParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "param", param_idx, "max"] => {
                Some(TrackFXParamParams {
                    track_guid: track_guid.to_string(),
                    fx_idx: fx_idx.parse().ok()?,
                    param_idx: param_idx.parse().ok()?,
                })
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Bundle(OscBundle {
            timetag: OscTime::try_from(SystemTime::now()).unwrap(),
            content: vec![
                OscPacket::Message(OscMessage {
                    addr: format!(
                        "/track/{}/fx/{}/param/{}/max",
                        args.track_guid, args.fx_idx, args.param_idx
                    ),
                    args: vec![OscType::Float(args.param_info.param_max as f32)],
                }),
                // TODO: step size?
            ],
        })]
    }

    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        collect_track_param_info(params, reaper).unwrap();
        Ok(TrackFXParamArgs {
            track_guid: params.track_guid.clone(),
            fx_idx: params.fx_idx,
            param_idx: params.param_idx,
            param_info: collect_track_param_info(params, reaper)?,
        })
    }
}

/// @osc-doc
/// @queryable
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/info
/// - params:
///   - track_guid (string): unique identifier for the track
///   - fx_idx (int): index of the FX on the track
///
/// Replies with many OSC messages reporting the folowing:
///   guid (string): unique identifier for the FX
///   name (string): name of the FX
///   param_count (int): number of parameters for the FX
///   for each param:
///     param_idx (int): index of the parameter
///     param_name (string): name of the parameter
///     param_value (float): current value of the parameter, normalized to 0.
///     param_min (float): minimum value of the parameter, normalized to 0. TODO: what here?
///     param_max (float): maximum value of the parameter, normalized to 1.0
pub struct TrackFXInfoRoute;
pub struct TrackFXInfoParams {
    track_guid: String,
    fx_idx: u32,
}
pub struct TrackFXInfoArgs {
    track_guid: String,
    fx_idx: u32,
    fx_info: FxInfo,
}

impl OscRoute for TrackFXInfoRoute {
    type SendParams = TrackFXInfoArgs;
    type ReceiveParams = TrackFXInfoParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["track", track_guid, "fx", fx_idx, "info"] => Some(TrackFXInfoParams {
                track_guid: track_guid.to_string(),
                fx_idx: fx_idx.parse().ok()?,
            }),
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        let mut packets = vec![];
        packets.extend(TrackFXGuidRoute::build_packets(
            fx_guid_to_string(args.fx_info.guid),
            _reaper,
        ));
        packets.extend(TrackFXNameRoute::build_packets(
            args.fx_info.name.clone(),
            _reaper,
        ));
        packets.extend(TrackFXParamCountRoute::build_packets(
            args.fx_info.num_params,
            _reaper,
        ));
        for (param_idx, fx_param) in args.fx_info.params.iter().enumerate() {
            packets.extend(TrackFXParamNameRoute::build_packets(
                TrackFXParamArgs {
                    track_guid: args.track_guid.clone(),
                    fx_idx: args.fx_idx,
                    param_idx: param_idx as u32,
                    param_info: fx_param.clone(),
                },
                _reaper,
            ));
            packets.extend(TrackFXParamValueRoute::build_packets(
                TrackFXParamArgs {
                    track_guid: args.track_guid.clone(),
                    fx_idx: args.fx_idx,
                    param_idx: param_idx as u32,
                    param_info: fx_param.clone(),
                },
                _reaper,
            ));
            packets.extend(TrackFXParamMinRoute::build_packets(
                TrackFXParamArgs {
                    track_guid: args.track_guid.clone(),
                    fx_idx: args.fx_idx,
                    param_idx: param_idx as u32,
                    param_info: fx_param.clone(),
                },
                _reaper,
            ));
            packets.extend(TrackFXParamMaxRoute::build_packets(
                TrackFXParamArgs {
                    track_guid: args.track_guid.clone(),
                    fx_idx: args.fx_idx,
                    param_idx: param_idx as u32,
                    param_info: fx_param.clone(),
                },
                _reaper,
            ));
        }
        packets
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
            let fx_guid = reaper
                .track_fx_get_fx_guid(
                    track,
                    reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
                )
                .unwrap();
            let num_params = reaper.track_fx_get_num_params(
                track,
                reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
            );
            let enabled = reaper.track_fx_get_enabled(
                track,
                reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
            );
            let mut fx_params = vec![];
            for param_idx in 0..num_params {
                fx_params.push(
                    collect_track_param_info(
                        &TrackFXParamParams {
                            track_guid: params.track_guid.clone(),
                            fx_idx: params.fx_idx,
                            param_idx,
                        },
                        reaper,
                    )
                    .unwrap(),
                );
            }
            Ok(Self::SendParams {
                track_guid: params.track_guid.clone(),
                fx_idx: params.fx_idx,
                fx_info: FxInfo {
                    name: fx_name.to_string(),
                    guid: fx_guid,
                    enabled,
                    num_params,
                    params: fx_params,
                },
            })
        }
    }
}

fn collect_track_param_info(
    params: &TrackFXParamParams,
    reaper: &Reaper,
) -> Result<ParamInfo, RouteError> {
    let track = get_track_by_guid(reaper, &params.track_guid)?;
    let param_name = unsafe {
        reaper
            .track_fx_get_param_name(
                track,
                reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
                params.param_idx,
                128, // Name length in bytes TODO: what size makes sense here?
            )
            .unwrap()
    };
    let param_ex = unsafe {
        reaper.track_fx_get_param_ex(
            track,
            reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
            params.param_idx,
        )
    };
    let param_step_size = unsafe {
        reaper.track_fx_get_parameter_step_sizes(
            track,
            reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
            params.param_idx,
        )
    };
    Ok(ParamInfo {
        param_name: param_name.to_string(),
        param_value: param_ex.current_value,
        param_min: param_ex.min_value,
        param_max: param_ex.max_value,
        param_step_size: None, // TODO
    })
}

/// @osc-doc
/// @readable
/// OSC Address: /fxinfo/{ident}/name
/// - params:
///   - ident (string): unique identifier for the FX
/// - args:
///   - name (string): name of the FX
pub struct FxInfoNameRoute;
impl OscRoute for FxInfoNameRoute {
    type SendParams = String;
    type ReceiveParams = String; // ident

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["fxinfo", ident, "name"] => Some(ident.to_string()),
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/fxinfo/{ident}/name".to_string(),
            args: vec![OscType::String(args)],
        })]
    }

    fn collect_send_params(
        _params: &Self::ReceiveParams,
        _reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        Ok("".to_string())
    }
}
///
/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /fxinfo/{ident}/param_count
/// - params:
///   - ident (string): unique identifier for the FX
/// - args:
///   - param_count (int): number of parameters for the FX
pub struct FxInfoParamCountRoute;

impl OscRoute for FxInfoParamCountRoute {
    type SendParams = u32;
    type ReceiveParams = String; // ident

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["fxinfo", ident, "param_count"] => Some(ident.to_string()),
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/fxinfo/{ident}/param_count".to_string(),
            args: vec![OscType::Int(args as i32)],
        })]
    }

    fn collect_send_params(
        _params: &Self::ReceiveParams,
        _reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        Ok(0)
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /fxinfo/{ident}/param/{param_idx}/name
/// - params:
///   - ident (string): unique identifier for the FX
///   - param_idx (int): index of the parameter
/// - args:
///   - param_name (string): name of the parameter
pub struct FxInfoParamNameRoute;

impl OscRoute for FxInfoParamNameRoute {
    type SendParams = String;
    type ReceiveParams = (String, u32); // (ident, param_idx)

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["fxinfo", ident, "param", param_idx, "name"] => {
                Some((ident.to_string(), param_idx.parse().ok()?))
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/fxinfo/{ident}/param/{param_idx}/name".to_string(),
            args: vec![OscType::String(args)],
        })]
    }

    fn collect_send_params(
        _params: &Self::ReceiveParams,
        _reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        Ok("".to_string())
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /fxinfo/{ident}/param/{param_idx}/min
/// - params:
///   - ident (string): unique identifier for the FX
///   - param_idx (int): index of the parameter
/// - args:
///   - param_min (float): minimum raw value of the parameter
pub struct FxInfoParamMinRoute;

impl OscRoute for FxInfoParamMinRoute {
    type SendParams = f64;
    type ReceiveParams = (String, u32); // (ident, param_idx)

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["fxinfo", ident, "param", param_idx, "min"] => {
                Some((ident.to_string(), param_idx.parse().ok()?))
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/fxinfo/{ident}/param/{param_idx}/min".to_string(),
            args: vec![OscType::Float(args as f32)],
        })]
    }

    fn collect_send_params(
        _params: &Self::ReceiveParams,
        _reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        Ok(0.0)
    }
}

/// @osc-doc
/// @readable
/// @queryable
/// OSC Address: /fxinfo/{ident}/param/{param_idx}/max
/// - params:
///   - ident (string): unique identifier for the FX
///   - param_idx (int): index of the parameter
/// - args:
///   - param_max (float): maximum raw value of the parameter
pub struct FxInfoParamMaxRoute;

impl OscRoute for FxInfoParamMaxRoute {
    type SendParams = f64;
    type ReceiveParams = (String, u32); // (ident, param_idx)

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams> {
        match segments {
            ["fxinfo", ident, "param", param_idx, "max"] => {
                Some((ident.to_string(), param_idx.parse().ok()?))
            }
            _ => None,
        }
    }

    fn build_packets(args: Self::SendParams, _reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Message(OscMessage {
            addr: "/fxinfo/{ident}/param/{param_idx}/max".to_string(),
            args: vec![OscType::Float(args as f32)],
        })]
    }

    fn collect_send_params(
        _params: &Self::ReceiveParams,
        _reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError> {
        Ok(0.0)
    }
}

/// @osc-doc
/// @queryable
/// OSC Address: /fxinfo
/// Replies with many OSC messages reporting the following for all FX on all tracks:
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
                let guid = fx_guid_to_string(fx_info.guid);
                let mut msg_content = vec![];
                msg_content.extend(FxInfoNameRoute::build_packets(
                    fx_info.name.clone(),
                    _reaper,
                ));
                msg_content.extend(FxInfoParamCountRoute::build_packets(
                    fx_info.num_params,
                    _reaper,
                ));
                messages.push(OscPacket::Bundle(OscBundle {
                    timetag: OscTime::try_from(SystemTime::now()).unwrap(),
                    content: msg_content,
                }));
                for (param_idx, param) in fx_info.params.clone().into_iter().enumerate() {
                    let mut msg_content = vec![];
                    msg_content.extend(FxInfoParamNameRoute::build_packets(
                        param.param_name.clone(),
                        _reaper,
                    ));
                    msg_content
                        .extend(FxInfoParamMinRoute::build_packets(param.param_min, _reaper));
                    msg_content
                        .extend(FxInfoParamMaxRoute::build_packets(param.param_max, _reaper));
                    messages.push(OscPacket::Bundle(OscBundle {
                        timetag: OscTime::try_from(SystemTime::now()).unwrap(),
                        content: msg_content,
                    }));
                }
                messages
            })
            .collect()
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
                let fx_index = reaper
                    .track_fx_add_by_name_add(
                        track,
                        name.clone(),
                        reaper_medium::TrackFxChainType::NormalFxChain,
                        reaper_medium::AddFxBehavior::AddIfNotFound,
                    )
                    .unwrap();
                let fx_guid = reaper
                    .track_fx_get_fx_guid(
                        track,
                        reaper_medium::TrackFxLocation::NormalFxChain(fx_index),
                    )
                    .unwrap();
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
                        .unwrap();
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
                    params.push(ParamInfo {
                        param_name: param_name.into_inner().to_string_lossy().to_string(), //TODO: fixme
                        param_value: param_ex.current_value,
                        param_min: param_ex.min_value,
                        param_max: param_ex.max_value,
                        param_step_size: None, // TODO
                    });
                }
                // println!("FX {}: {} ({})", i, name, ident);
                fx_infos.push(FxInfo {
                    name,
                    guid: fx_guid,
                    enabled: false,
                    num_params,
                    params,
                });
            }
            i += 1;
        }
        unsafe { reaper.delete_track(track) };
        Ok(AllFxInfoArgs { fx: fx_infos })
    }
}

use std::ffi::{CStr, CString};
use std::time::SystemTime;

use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};

use crate::registries::hash_fx_ident;
use crate::{get_track_by_guid, OscRoute, Reaper, ReceiverError, RouteError};

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
/// - ident (int): unique identifier for the FX
/// - name (string): name of the FX
/// - param_count (int): number of parameters for the FX
/// - for each param:
///   - param_idx (int): index of the parameter
///   - param_name (string): name of the parameter
///   - param_value (float): current value of the parameter, normalized to 0.
///   - param_min (float): minimum value of the parameter, normalized to 0. TODO: what here?
///   - param_max (float): maximum value of the parameter, normalized to 1.0
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
    pub param_step_size: Option<f64>,
}
#[derive(Clone)]
pub struct FxInfo {
    pub name: String,
    pub ident: String,
    pub num_params: u32,
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
                    num_params: reaper.track_fx_get_num_params(
                        track,
                        reaper_medium::TrackFxLocation::NormalFxChain(params.fx_idx),
                    ),
                    params: vec![],
                },
            })
        }
    }
}

/// @osc-doc
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param/{param_idx}/value
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - fx_idx (int): index of the FX on the track
/// - param_idx (int): index of the parameter
/// - value (float): value of the parameter
///
/// @osc-doc
/// @readonly
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param/{param_idx}/min
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - fx_idx (int): index of the FX on the track
/// - param_idx (int): index of the parameter
/// - min (float): minimum value of the parameter
///
/// @osc-doc
/// @readonly
/// OSC Address: /track/{track_guid}/fx/{fx_idx}/param/{param_idx}/max
/// Arguments:
/// - track_guid (string): unique identifier for the track
/// - fx_idx (int): index of the FX on the track
/// - param_idx (int): index of the parameter
/// - max (float): maximum value of the parameter
pub struct TrackFXParamRoute;
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
impl OscRoute for TrackFXParamRoute {
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
            // TODO: should not match these for the read case
            ["track", track_guid, "fx", fx_idx, "param", param_idx, "min"] => {
                Some(TrackFXParamParams {
                    track_guid: track_guid.to_string(),
                    fx_idx: fx_idx.parse().ok()?,
                    param_idx: param_idx.parse().ok()?,
                })
            }
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

    fn receive(
        _params: Self::ReceiveParams,
        _msg: &OscMessage,
        _reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        Ok(())
    }

    fn build_packets(args: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket> {
        vec![OscPacket::Bundle(OscBundle {
            timetag: OscTime::try_from(SystemTime::now()).unwrap(),
            content: vec![
                OscPacket::Message(OscMessage {
                    addr: format!(
                        "/track/{}/fx/{}/param/{}/name",
                        args.track_guid, args.fx_idx, args.param_idx
                    ),
                    args: vec![OscType::String(args.param_info.param_name.clone())],
                }),
                OscPacket::Message(OscMessage {
                    addr: format!(
                        "/track/{}/fx/{}/param/{}/value",
                        args.track_guid, args.fx_idx, args.param_idx
                    ),
                    args: vec![OscType::Float(args.param_info.param_value as f32)],
                }),
                OscPacket::Message(OscMessage {
                    addr: format!(
                        "/track/{}/fx/{}/param/{}/min",
                        args.track_guid, args.fx_idx, args.param_idx
                    ),
                    args: vec![OscType::Float(args.param_info.param_min as f32)],
                }),
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
        Ok(TrackFXParamArgs {
            track_guid: params.track_guid.clone(),
            fx_idx: params.fx_idx,
            param_idx: params.param_idx,
            param_info: ParamInfo {
                param_name: param_name.to_string(),
                param_value: param_ex.current_value,
                param_min: param_ex.min_value,
                param_max: param_ex.max_value,
                param_step_size: None, // TODO
            },
        })
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
                let hashed_ident = hash_fx_ident(&fx_info.name);
                messages.push(OscPacket::Bundle(OscBundle {
                    timetag: OscTime::try_from(SystemTime::now()).unwrap(),
                    content: {
                        vec![
                            OscPacket::Message(OscMessage {
                                addr: format!("/fxinfo/{}/ident", hashed_ident).to_string(),
                                args: vec![OscType::String(fx_info.ident.clone())],
                            }),
                            OscPacket::Message(OscMessage {
                                addr: format!("/fxinfo/{}/name", hashed_ident).to_string(),
                                args: vec![OscType::String(fx_info.name.clone())],
                            }),
                            OscPacket::Message(OscMessage {
                                addr: format!("/fxinfo/{}/param_count", hashed_ident).to_string(),
                                args: vec![OscType::Int(fx_info.num_params as i32)],
                            }),
                        ]
                    },
                }));
                for (param_idx, param) in fx_info.params.clone().into_iter().enumerate() {
                    messages.push(OscPacket::Bundle(OscBundle {
                        timetag: OscTime::try_from(SystemTime::now()).unwrap(),
                        content: vec![
                            OscPacket::Message(OscMessage {
                                addr: format!("/fxinfo/{}/param/{}/name", hashed_ident, param_idx),
                                args: vec![OscType::String(param.param_name.clone())],
                            }),
                            // messages.push(OscPacket::Message(OscMessage {
                            //     addr: format!("/fxinfo/{}/param/{}/value", ident, param_idx),
                            //     args: vec![OscType::Float(param.param_value as f32)],
                            // }));
                            OscPacket::Message(OscMessage {
                                addr: format!("/fxinfo/{}/param/{}/min", hashed_ident, param_idx),
                                args: vec![OscType::Float(param.param_min as f32)],
                            }),
                            OscPacket::Message(OscMessage {
                                addr: format!("/fxinfo/{}/param/{}/max", hashed_ident, param_idx),
                                args: vec![OscType::Float(param.param_max as f32)],
                            }),
                            // messages.push(OscPacket::Message(OscMessage {
                            //     addr: format!("/fxinfo/{}/param/{}/default", ident, param_idx),
                            //     args: vec![OscType::Float(param.param_default as f32)],
                            // }));
                        ],
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
                    ident,
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

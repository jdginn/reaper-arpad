use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::OnceLock;

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::ProjectContext::CurrentProject;
use reaper_medium::{
    ControlSurface, Reaper, ReaperFunctionError, ReaperSession, TrackAttributeKey,
};

use fragile::Fragile;

use rosc::{encoder, OscMessage, OscPacket};

use crossbeam_channel::{bounded, Receiver, Sender};
use std::thread;

mod utils;
use utils::{get_track_by_guid, get_track_guid, get_track_idx};

mod registries;

mod track_routes;
use track_routes::*;

mod fx_routes;
use fx_routes::*;

mod polling;
use polling::*;

#[derive(Debug)]
pub enum RouteError {
    GuidNotFound(String),
    ValueNotFound(String),
}

#[derive(Debug)]
pub enum ReceiverError {
    Route(RouteError),
    BadValue(String),
    Reaper(reaper_medium::ReaperFunctionError),
}

impl From<RouteError> for ReceiverError {
    fn from(e: RouteError) -> Self {
        ReceiverError::Route(e)
    }
}

impl From<reaper_medium::ReaperFunctionError> for ReceiverError {
    fn from(e: ReaperFunctionError) -> Self {
        ReceiverError::Reaper(e)
    }
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteError::GuidNotFound(guid) => write!(f, "GUID not found: {}", guid),
            RouteError::ValueNotFound(value) => write!(f, "Value not found: {}", value),
        }
    }
}

pub(crate) trait OscRoute {
    type SendParams;
    type ReceiveParams;

    fn matcher(_segments: &[&str]) -> Option<Self::ReceiveParams> {
        // Default case is to do nothing if not @writeable or @queryable
        return None;
    }
    fn receive(
        _params: Self::ReceiveParams,
        _msg: &OscMessage,
        _reaper: &Reaper,
    ) -> Result<(), ReceiverError> {
        // Default case is do nothing if not @writeable
        Ok(())
    }
    fn build_packets(params: Self::SendParams, reaper: &Reaper) -> Vec<OscPacket>;

    /// Given receive params and reaper, build the corresponding SendParams for query
    fn collect_send_params(
        params: &Self::ReceiveParams,
        reaper: &Reaper,
    ) -> Result<Self::SendParams, RouteError>;
}

fn dispatch_route<T: OscRoute>(
    segments: &[&str],
    msg: &OscMessage,
    reaper: &Reaper,
    osc_sender: &Sender<OscPacket>,
) {
    let is_query = segments.last() == Some(&"?");
    let match_segments = if is_query {
        &segments[..segments.len() - 1]
    } else {
        segments
    };

    if let Some(params) = T::matcher(match_segments) {
        if is_query {
            match T::collect_send_params(&params, reaper) {
                Ok(send_params) => {
                    T::build_packets(send_params, reaper)
                        .into_iter()
                        .for_each(|packet| {
                            osc_sender.send(packet).unwrap();
                        });
                }
                Err(e) => {
                    eprintln!("Query failed: {:?}", e);
                }
            }
        } else {
            T::receive(params, msg, reaper).unwrap_or_else(|e| {
                eprintln!("Receive failed: {:?}", e);
            });
        }
    }
}

struct ArpadSurface {
    osc_sender: Sender<OscPacket>,
    sock: UdpSocket,
    reaper: Reaper,
    poll_manager: PollManager,
    known_guids: RefCell<HashSet<String>>,
}

impl std::fmt::Debug for ArpadSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ArpadSurface")
            .field("osc_sender", &"...")
            .field("sock", &"...")
            .field("reaper", &"...")
            .field("poll_manager", &"[PollManager omitted]")
            .finish()
    }
}

impl ControlSurface for ArpadSurface {
    fn set_track_list_change(&self) {
        let mut temp_guids = HashSet::new();
        for i in 0..self.reaper.count_tracks(CurrentProject) {
            let track = self.reaper.get_track(CurrentProject, i).unwrap();
            let guid = get_track_guid(&self.reaper, track);
            temp_guids.insert(guid.clone());
            let track_idx = get_track_idx(&self.reaper, track);
            TrackIndexRoute::build_packets(
                TrackIndexArgs {
                    track,
                    index: track_idx as i32,
                },
                &self.reaper,
            )
            .into_iter()
            .for_each(|packet| {
                self.osc_sender.send(packet).unwrap();
            });
            unsafe {
                for i in 0..self
                    .reaper
                    .get_track_num_sends(track, reaper_medium::TrackSendCategory::Send)
                {
                    let dest = self
                        .reaper
                        .get_track_send_info_desttrack(
                            track,
                            reaper_medium::TrackSendDirection::Send,
                            i,
                        )
                        .unwrap();
                    TrackSendGuidRoute::build_packets(
                        TrackSendGuidArgs {
                            track,
                            send_index: i as i32,
                            send_guid: get_track_guid(&self.reaper, dest),
                        },
                        &self.reaper,
                    )
                    .into_iter()
                    .for_each(|packet| {
                        self.osc_sender.send(packet).unwrap();
                    });
                }
            }
        }
        let mut known_guids = self.known_guids.borrow_mut();
        for guid in known_guids.difference(&temp_guids) {
            TrackDeleteRoute::build_packets(
                TrackDeleteArgs {
                    track_guid: guid.clone(),
                },
                &self.reaper,
            )
            .into_iter()
            .for_each(|packet| {
                self.osc_sender.send(packet).unwrap();
            });
        }
        known_guids.clear();
        known_guids.extend(temp_guids.iter().cloned());
    }
    // This is also called when track color changes!
    fn set_track_title(&self, args: reaper_medium::SetTrackTitleArgs) {
        TrackNameRoute::build_packets(
            TrackNameArgs {
                track: args.track,
                name: args.name.to_string(),
            },
            &self.reaper,
        )
        .into_iter()
        .for_each(|packet| {
            self.osc_sender.send(packet).unwrap();
        });
        let color = unsafe {
            self.reaper
                .get_set_media_track_info_get_custom_color(args.track)
                .color
        };
        TrackColorRoute::build_packets(
            TrackColorArgs {
                track: args.track,
                color: color.to_raw(),
            },
            &self.reaper,
        )
        .into_iter()
        .for_each(|packet| {
            self.osc_sender.send(packet).unwrap();
        });
    }
    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        TrackVolumeRoute::build_packets(args, &self.reaper)
            .into_iter()
            .for_each(|packet| {
                self.osc_sender.send(packet).unwrap();
            });
    }
    fn set_surface_pan(&self, args: reaper_medium::SetSurfacePanArgs) {
        TrackPanRoute::build_packets(args, &self.reaper)
            .into_iter()
            .for_each(|packet| {
                self.osc_sender.send(packet).unwrap();
            });
    }
    fn set_surface_mute(&self, args: reaper_medium::SetSurfaceMuteArgs) {
        TrackMuteRoute::build_packets(args, &self.reaper)
            .into_iter()
            .for_each(|packet| {
                self.osc_sender.send(packet).unwrap();
            });
    }
    fn ext_set_fx_param(&self, args: reaper_medium::ExtSetFxParamArgs) -> i32 {
        TrackFXParamValueRoute::build_packets(
            TrackFXParamArgs {
                track_guid: get_track_guid(&self.reaper, args.track),
                fx_idx: args.fx_index,
                param_idx: args.param_index,
                param_info: ParamInfo {
                    param_value: args.param_value.get(),
                    ..Default::default()
                },
            },
            &self.reaper,
        )
        .into_iter()
        .for_each(|packet| {
            self.osc_sender.send(packet).unwrap();
        });
        0 // TODO: is this correct?
    }
    // TODO: implement
    // fn ext_set_fx_enabled(&self, args: reaper_medium::ExtSetFxEnabledArgs) {
    //     TrackFXEnabledRoute::build_packets(
    //         TrackFXInfoArgs {
    //             track_guid: get_track_guid(&self.reaper, args.track),
    //             fx_idx: args.fx_location,
    //             enabled: args.enabled,
    //         },
    //         &self.reaper,
    //     )
    //     .into_iter()
    //     .for_each(|packet| {
    //         self.osc_sender.send(packet).unwrap();
    //     });
    // }
    fn run(&mut self) {
        self.poll_manager.poll_all(&self.osc_sender);
        let mut buf = [0u8; rosc::decoder::MTU];
        loop {
            match self.sock.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    if let Ok((_addr, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                        handle_packet(self.reaper.clone(), packet, &self.osc_sender);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, exit loop
                    break;
                }
                Err(e) => {
                    eprintln!("OSC receive error: {:?}", e);
                    break;
                }
            }
        }
    }
}

// Spawn the OSC sending thread
fn start_sender_thread(dev_addr: SocketAddrV4, sock: UdpSocket, osc_receiver: Receiver<OscPacket>) {
    thread::spawn(move || {
        for msg in osc_receiver.iter() {
            if let Ok(buf) = encoder::encode(&msg) {
                match sock.send_to(buf.as_slice(), dev_addr) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("OSC send error: {:?}", e);
                    }
                }
            }
        }
    });
}

fn parse_osc_address(addr: &str) -> Vec<&str> {
    addr.split('/').filter(|s| !s.is_empty()).collect()
}

fn handle_packet(reaper: Reaper, packet: OscPacket, osc_sender: &Sender<OscPacket>) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC message: {:?}", msg);
            let segments = parse_osc_address(&msg.addr);
            dispatch_route::<TrackNameRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackSelectedRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackVolumeRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackPanRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackMuteRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackSoloRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackRecArmRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackSendVolumeRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackSendPanRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackColorRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXGuidRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXNameRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXEnabledRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXParamCountRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXParamNameRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXParamValueRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXParamMinRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXParamMaxRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<TrackFXInfoRoute>(&segments, &msg, &reaper, osc_sender);
            dispatch_route::<AllFxInfoRoute>(&segments, &msg, &reaper, osc_sender);
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC bundle: {:?}", bundle);
        }
    }
}

const HOST_ADDR: &str = "0.0.0.0:9090";
const DEVICE_ADDR: &str = "0.0.0.0:9091";

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
        SocketAddrV4::from_str(arg).unwrap()
    }
    let host_addr = get_addr_from_arg(HOST_ADDR);
    let dev_addr = get_addr_from_arg(DEVICE_ADDR);
    let sock = UdpSocket::bind(host_addr).unwrap();
    sock.set_nonblocking(true)?;
    let (osc_sender, osc_receiver) = bounded(128); // buffer size as needed
    start_sender_thread(dev_addr, sock.try_clone().unwrap(), osc_receiver);

    let mut session = reaper_medium::ReaperSession::load(context);
    let reaper = session.reaper().clone();
    let mut poll_manager = PollManager::new();
    // poll_manager.add_source(Box::new(TrackColorPollSource::new(reaper.clone())));
    //  TODO: add various polling sources here
    let mut arpad = ArpadSurface {
        sock,
        osc_sender,
        reaper: reaper.clone(),
        poll_manager,
        known_guids: RefCell::new(HashSet::new()),
    };
    arpad.run();
    match session.plugin_register_add_csurf_inst(Box::new(arpad)) {
        Ok(_) => {}
        Err(_) => {
            println!("Failed to load csurf");
        }
    }
    let _ = REAPER_SESSION.set(Fragile::new(session));

    Ok(())
}
static REAPER_SESSION: OnceLock<Fragile<ReaperSession>> = OnceLock::new();

use fragile::Fragile;
use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::ProjectContext::CurrentProject;
use reaper_medium::ReaperFunctionError;
use reaper_medium::{ControlSurface, MediaTrack, Reaper, ReaperSession, TrackAttributeKey};
use std::error::Error;
use std::sync::OnceLock;

use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

use rosc::encoder;
use rosc::{OscMessage, OscPacket};

use crossbeam_channel::{bounded, Receiver, Sender};
use std::thread;

mod osc_routes;
use osc_routes::*;

fn guid_to_string(guid: reaper_low::raw::GUID) -> String {
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

fn get_track_idx(reaper: &Reaper, track: MediaTrack) -> u32 {
    unsafe { reaper.get_media_track_info_value(track, TrackAttributeKey::TrackNumber) as u32 }
}

fn get_track_guid(reaper: &Reaper, track: MediaTrack) -> String {
    unsafe {
        let track_id = reaper.get_set_media_track_info_get_guid(track);
        guid_to_string(track_id)
    }
}

fn get_track_by_guid(reaper: &Reaper, guid: &str) -> Result<MediaTrack, RouteError> {
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
            // ...other error formatting
        }
    }
}

trait OscRoute {
    type SendParams;
    type ReceiveParams;

    fn matcher(segments: &[&str]) -> Option<Self::ReceiveParams>;
    fn receive(
        params: Self::ReceiveParams,
        msg: &OscMessage,
        reaper: &Reaper,
    ) -> Result<(), ReceiverError>;
    fn build_message(params: Self::SendParams, reaper: &Reaper) -> OscMessage;
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
                    let response_msg = T::build_message(send_params, reaper);
                    osc_sender.send(OscPacket::Message(response_msg)).unwrap();
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

#[derive(Debug)]
struct ArpadSurface {
    osc_sender: Sender<OscPacket>,
    sock: UdpSocket,
    reaper: Reaper,
}

impl ArpadSurface {
    fn send(&self, msg: OscMessage) {
        self.osc_sender.send(OscPacket::Message(msg)).unwrap();
    }
}

impl ControlSurface for ArpadSurface {
    fn set_track_list_change(&self) {
        for i in 0..self.reaper.count_tracks(CurrentProject) {
            let track = self.reaper.get_track(CurrentProject, i).unwrap();
            let track_idx = get_track_idx(&self.reaper, track);
            self.osc_sender
                .send(OscPacket::Message(TrackIndexRoute::build_message(
                    TrackIndexArgs {
                        track,
                        index: track_idx as i32,
                    },
                    &self.reaper,
                )))
                .unwrap();
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
                    self.osc_sender
                        .send(OscPacket::Message(TrackSendGuidRoute::build_message(
                            TrackSendGuidArgs {
                                track,
                                send_index: i as i32,
                                send_guid: get_track_guid(&self.reaper, dest),
                            },
                            &self.reaper,
                        )))
                        .unwrap();
                }
            }
        }
    }
    fn set_track_title(&self, args: reaper_medium::SetTrackTitleArgs) {
        self.send(osc_routes::TrackNameRoute::build_message(
            TrackNameArgs {
                track: args.track,
                name: args.name.to_string(),
            },
            &self.reaper,
        ));
    }
    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        self.send(osc_routes::TrackVolumeRoute::build_message(
            args,
            &self.reaper,
        ));
    }
    fn set_surface_pan(&self, args: reaper_medium::SetSurfacePanArgs) {
        self.send(osc_routes::TrackPanRoute::build_message(args, &self.reaper));
    }
    fn set_surface_mute(&self, args: reaper_medium::SetSurfaceMuteArgs) {
        self.send(osc_routes::TrackMuteRoute::build_message(
            args,
            &self.reaper,
        ));
    }
    fn run(&mut self) {
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
                let _ = sock.send_to(buf.as_slice(), dev_addr);
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
    let mut arpad = ArpadSurface {
        sock,
        osc_sender,
        reaper: reaper.clone(),
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

use rosc::{encoder, OscMessage, OscPacket, OscType};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::time::Duration;

const HOST_ADDR: &str = "0.0.0.0:9090";
const DEVICE_ADDR: &str = "0.0.0.0:9091";
const TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FxParam {
    name: String,
    index: u32,
    min: f32,
    max: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FxInfo {
    fx_name: String,
    params: Vec<FxParam>,
}

fn main() {
    println!("FX Dump Tool - Collecting FX information from Reaper via OSC");

    // Set up UDP sockets
    let host_addr = SocketAddrV4::from_str(HOST_ADDR)
        .expect("Failed to parse HOST_ADDR");
    let dev_addr = SocketAddrV4::from_str(DEVICE_ADDR)
        .expect("Failed to parse DEVICE_ADDR");

    // Bind to HOST_ADDR (where we receive messages)
    let sock = UdpSocket::bind(host_addr)
        .expect("Failed to bind to HOST_ADDR");

    println!("Bound to {}", host_addr);
    println!("Sending OSC query to Reaper at {}", dev_addr);

    // Send /fxinfo query message
    let query_msg = OscPacket::Message(OscMessage {
        addr: "/fxinfo/?".to_string(),
        args: vec![],
    });

    let msg_buf = encoder::encode(&query_msg)
        .expect("Failed to encode OSC message");

    sock.send_to(&msg_buf, dev_addr)
        .expect("Failed to send OSC message");

    println!("Query sent, waiting for responses...");

    // Set socket timeout to prevent indefinite blocking on recv
    // The manual timeout check below ensures we exit after TIMEOUT_SECS total elapsed time
    sock.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .expect("Failed to set socket timeout");

    // Collect FX information
    // Note: We use a Vec instead of HashMap because OSC addresses have literal placeholders
    // (e.g., "/fxinfo/{ident}/name") that don't get substituted, so all FX would have the
    // same key and overwrite each other. Instead, we track by order of arrival.
    let mut fx_list: Vec<FxInfo> = Vec::new();
    let mut current_fx: Option<FxInfo> = None;
    let mut buf = [0u8; rosc::decoder::MTU];

    let start_time = std::time::Instant::now();
    let mut message_count = 0;

    // Listen for incoming OSC messages
    loop {
        if start_time.elapsed() > Duration::from_secs(TIMEOUT_SECS) {
            println!("Timeout reached after {} seconds", TIMEOUT_SECS);
            break;
        }

        match sock.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                    message_count += 1;
                    process_packet(&packet, &mut fx_list, &mut current_fx);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout on recv - check if we've waited long enough
                continue;
            }
            Err(e) => {
                panic!("OSC receive error: {:?}", e);
            }
        }
    }

    // Add the last FX if it exists
    if let Some(fx) = current_fx {
        fx_list.push(fx);
    }

    println!("Received {} OSC messages", message_count);
    println!("Collected information for {} FX", fx_list.len());

    // Sort for consistent output
    fx_list.sort_by(|a, b| a.fx_name.cmp(&b.fx_name));

    // Write to YAML file
    let yaml_output = serde_yaml::to_string(&fx_list)
        .expect("Failed to serialize to YAML");

    std::fs::write("fx_dump.yaml", yaml_output)
        .expect("Failed to write YAML file");

    println!("FX information written to fx_dump.yaml");
}

fn process_packet(
    packet: &OscPacket,
    fx_list: &mut Vec<FxInfo>,
    current_fx: &mut Option<FxInfo>,
) {
    match packet {
        OscPacket::Message(msg) => {
            process_message(msg, fx_list, current_fx);
        }
        OscPacket::Bundle(bundle) => {
            for content in &bundle.content {
                process_packet(content, fx_list, current_fx);
            }
        }
    }
}

fn ensure_param_exists(fx_entry: &mut FxInfo, param_idx: u32) -> &mut FxParam {
    if let Some(pos) = fx_entry.params.iter().position(|p| p.index == param_idx) {
        &mut fx_entry.params[pos]
    } else {
        let new_index = fx_entry.params.len();
        fx_entry.params.push(FxParam {
            name: String::new(),
            index: param_idx,
            min: 0.0,
            max: 1.0,
        });
        &mut fx_entry.params[new_index]
    }
}

fn process_message(
    msg: &OscMessage,
    fx_list: &mut Vec<FxInfo>,
    current_fx: &mut Option<FxInfo>,
) {
    let segments: Vec<&str> = msg.addr.split('/').filter(|s| !s.is_empty()).collect();

    match segments.as_slice() {
        ["fxinfo", _fx_ident, "name"] => {
            // When we receive a new FX name, save the previous FX and start a new one
            if let Some(OscType::String(name)) = msg.args.first() {
                if let Some(fx) = current_fx.take() {
                    fx_list.push(fx);
                }
                *current_fx = Some(FxInfo {
                    fx_name: name.clone(),
                    params: vec![],
                });
            }
        }
        ["fxinfo", _fx_ident, "param_count"] => {
            // We don't need to track param_count - we collect params as they arrive
        }
        ["fxinfo", _fx_ident, "param", param_idx_str, "name"] => {
            if let Ok(param_idx) = param_idx_str.parse::<u32>() {
                if let Some(OscType::String(param_name)) = msg.args.first() {
                    if let Some(fx) = current_fx.as_mut() {
                        let param = ensure_param_exists(fx, param_idx);
                        param.name = param_name.clone();
                    }
                }
            }
        }
        ["fxinfo", _fx_ident, "param", param_idx_str, "min"] => {
            if let Ok(param_idx) = param_idx_str.parse::<u32>() {
                if let Some(OscType::Float(min_val)) = msg.args.first() {
                    if let Some(fx) = current_fx.as_mut() {
                        let param = ensure_param_exists(fx, param_idx);
                        param.min = *min_val;
                    }
                }
            }
        }
        ["fxinfo", _fx_ident, "param", param_idx_str, "max"] => {
            if let Ok(param_idx) = param_idx_str.parse::<u32>() {
                if let Some(OscType::Float(max_val)) = msg.args.first() {
                    if let Some(fx) = current_fx.as_mut() {
                        let param = ensure_param_exists(fx, param_idx);
                        param.max = *max_val;
                    }
                }
            }
        }
        _ => {
            // Ignore other messages
        }
    }
}

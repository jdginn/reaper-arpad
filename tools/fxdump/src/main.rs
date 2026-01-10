use rosc::{encoder, OscMessage, OscPacket, OscType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
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

fn main() -> Result<(), Box<dyn Error>> {
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

    // Set socket timeout
    sock.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .expect("Failed to set socket timeout");

    // Collect FX information
    let mut fx_data: HashMap<String, FxInfo> = HashMap::new();
    let mut param_counts: HashMap<String, u32> = HashMap::new();
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
                    process_packet(&packet, &mut fx_data, &mut param_counts);
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

        // If we haven't received messages in a while, assume we're done
        if message_count > 0 && start_time.elapsed() > Duration::from_secs(TIMEOUT_SECS) {
            break;
        }
    }

    println!("Received {} OSC messages", message_count);
    println!("Collected information for {} FX", fx_data.len());

    // Convert to sorted vector for consistent output
    let mut fx_list: Vec<FxInfo> = fx_data.into_values().collect();
    fx_list.sort_by(|a, b| a.fx_name.cmp(&b.fx_name));

    // Write to YAML file
    let yaml_output = serde_yaml::to_string(&fx_list)
        .expect("Failed to serialize to YAML");

    std::fs::write("fx_dump.yaml", yaml_output)
        .expect("Failed to write YAML file");

    println!("FX information written to fx_dump.yaml");

    Ok(())
}

fn process_packet(
    packet: &OscPacket,
    fx_data: &mut HashMap<String, FxInfo>,
    param_counts: &mut HashMap<String, u32>,
) {
    match packet {
        OscPacket::Message(msg) => {
            process_message(msg, fx_data, param_counts);
        }
        OscPacket::Bundle(bundle) => {
            for content in &bundle.content {
                process_packet(content, fx_data, param_counts);
            }
        }
    }
}

fn process_message(
    msg: &OscMessage,
    fx_data: &mut HashMap<String, FxInfo>,
    param_counts: &mut HashMap<String, u32>,
) {
    let segments: Vec<&str> = msg.addr.split('/').filter(|s| !s.is_empty()).collect();

    match segments.as_slice() {
        ["fxinfo", fx_ident, "name"] => {
            if let Some(OscType::String(name)) = msg.args.first() {
                fx_data
                    .entry(fx_ident.to_string())
                    .or_insert_with(|| FxInfo {
                        fx_name: name.clone(),
                        params: vec![],
                    })
                    .fx_name = name.clone();
            }
        }
        ["fxinfo", fx_ident, "param_count"] => {
            if let Some(OscType::Int(count)) = msg.args.first() {
                param_counts.insert(fx_ident.to_string(), *count as u32);
            }
        }
        ["fxinfo", fx_ident, "param", param_idx_str, "name"] => {
            if let Ok(param_idx) = param_idx_str.parse::<u32>() {
                if let Some(OscType::String(param_name)) = msg.args.first() {
                    let fx_entry = fx_data
                        .entry(fx_ident.to_string())
                        .or_insert_with(|| FxInfo {
                            fx_name: String::new(),
                            params: vec![],
                        });

                    // Find or create param entry
                    if let Some(param) = fx_entry.params.iter_mut().find(|p| p.index == param_idx) {
                        param.name = param_name.clone();
                    } else {
                        fx_entry.params.push(FxParam {
                            name: param_name.clone(),
                            index: param_idx,
                            min: 0.0,
                            max: 1.0,
                        });
                    }
                }
            }
        }
        ["fxinfo", fx_ident, "param", param_idx_str, "min"] => {
            if let Ok(param_idx) = param_idx_str.parse::<u32>() {
                if let Some(OscType::Float(min_val)) = msg.args.first() {
                    let fx_entry = fx_data
                        .entry(fx_ident.to_string())
                        .or_insert_with(|| FxInfo {
                            fx_name: String::new(),
                            params: vec![],
                        });

                    if let Some(param) = fx_entry.params.iter_mut().find(|p| p.index == param_idx) {
                        param.min = *min_val;
                    } else {
                        fx_entry.params.push(FxParam {
                            name: String::new(),
                            index: param_idx,
                            min: *min_val,
                            max: 1.0,
                        });
                    }
                }
            }
        }
        ["fxinfo", fx_ident, "param", param_idx_str, "max"] => {
            if let Ok(param_idx) = param_idx_str.parse::<u32>() {
                if let Some(OscType::Float(max_val)) = msg.args.first() {
                    let fx_entry = fx_data
                        .entry(fx_ident.to_string())
                        .or_insert_with(|| FxInfo {
                            fx_name: String::new(),
                            params: vec![],
                        });

                    if let Some(param) = fx_entry.params.iter_mut().find(|p| p.index == param_idx) {
                        param.max = *max_val;
                    } else {
                        fx_entry.params.push(FxParam {
                            name: String::new(),
                            index: param_idx,
                            min: 0.0,
                            max: *max_val,
                        });
                    }
                }
            }
        }
        _ => {
            // Ignore other messages
        }
    }
}

use async_trait::async_trait;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::{DeviceDiscovery, DiscoveredDevice, ScanRequest};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Known camera MAC OUI prefixes and their brands
const CAMERA_OUI: &[(&str, &str)] = &[
    ("00:12:41", "V360 Pro"),
    ("00:18:AE", "V360 Pro"),
    ("28:6D:97", "V360 Pro"),
    ("38:22:D6", "V360"),
    ("A0:CC:2B", "V360"),
    ("7C:DD:E9", "V360"),
    ("48:02:2A", "Hikvision"),
    ("C0:56:E3", "Hikvision"),
    ("28:57:BE", "Hikvision"),
    ("80:48:A5", "Hikvision"),
    ("A4:14:37", "Dahua"),
    ("3C:EF:8C", "Dahua"),
    ("E0:50:8B", "Reolink"),
    ("EC:71:DB", "Reolink"),
    ("B0:A7:32", "TP-Link"),
    ("50:C7:BF", "TP-Link"),
    ("D8:0D:17", "TP-Link Tapo"),
    ("AC:84:C6", "TP-Link"),
    ("40:31:3C", "Yi Camera"),
    ("78:8B:2A", "Yi Camera"),
    ("34:CE:00", "Xiaomi"),
    ("64:09:80", "Xiaomi"),
    ("58:00:E3", "Tuya"),
    ("D4:A6:51", "Tuya"),
];

/// Common RTSP ports to probe
const _RTSP_PORTS: &[u16] = &[554, 8554, 8080, 80];

/// Common RTSP stream paths to try for different camera brands
const _RTSP_PATHS: &[(&str, &str)] = &[
    // V360 Pro common paths
    ("v360", "/live/ch00_1"),
    ("v360", "/ch0_0.h264"),
    ("v360", "/stream1"),
    // Hikvision
    ("hikvision", "/Streaming/Channels/101"),
    ("hikvision", "/ISAPI/Streaming/channels/101"),
    // Dahua
    ("dahua", "/cam/realmonitor?channel=1&subtype=0"),
    // Generic ONVIF
    ("generic", "/onvif/media_service/snapshot"),
    ("generic", "/stream"),
    ("generic", "/live"),
    ("generic", "/h264"),
    ("generic", "/media/video1"),
];

pub struct NetworkScanner;

impl NetworkScanner {
    pub fn new() -> Self {
        Self
    }

    /// Parse CIDR notation to get IP range
    fn parse_cidr(cidr: &str) -> Result<Vec<Ipv4Addr>, String> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid CIDR: {}", cidr));
        }

        let base_ip: Ipv4Addr = parts[0]
            .parse()
            .map_err(|_| format!("Invalid IP: {}", parts[0]))?;
        let prefix: u32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid prefix: {}", parts[1]))?;

        if prefix > 32 {
            return Err("Prefix must be 0-32".into());
        }

        let ip_u32 = u32::from(base_ip);
        let mask = if prefix == 0 {
            0
        } else {
            !((1u32 << (32 - prefix)) - 1)
        };
        let network = ip_u32 & mask;
        let broadcast = network | !mask;

        let mut ips = Vec::new();
        // Skip network and broadcast addresses, limit scan size
        let start = network + 1;
        let end = broadcast.min(network + 1024); // Max 1024 IPs per subnet

        for ip in start..end {
            ips.push(Ipv4Addr::from(ip));
        }

        Ok(ips)
    }

    /// Probe a single IP for camera services
    async fn probe_host(ip: Ipv4Addr, _scan_timeout: Duration) -> Option<DiscoveredDevice> {
        let mut found_ports = Vec::new();
        let mut protocols = Vec::new();
        let mut rtsp_url = None;

        // Probe common camera ports concurrently
        let ports_to_check = vec![554, 8554, 80, 8080, 443, 8443, 3702];

        for port in ports_to_check {
            let addr = SocketAddr::new(IpAddr::V4(ip), port);
            if let Ok(Ok(_)) =
                timeout(Duration::from_millis(500), TcpStream::connect(addr)).await
            {
                found_ports.push(port);
                debug!(ip = %ip, port = port, "Port open");
            }
        }

        if found_ports.is_empty() {
            return None;
        }

        // Determine protocols based on open ports
        if found_ports.contains(&554) || found_ports.contains(&8554) {
            protocols.push("rtsp".to_string());
            let rtsp_port = if found_ports.contains(&554) {
                554
            } else {
                8554
            };
            rtsp_url = Some(format!("rtsp://{}:{}/stream1", ip, rtsp_port));
        }
        if found_ports.contains(&80) || found_ports.contains(&8080) {
            protocols.push("http".to_string());
            // Could be MJPEG or ONVIF
        }
        if found_ports.contains(&3702) {
            protocols.push("onvif".to_string());
        }
        if found_ports.contains(&443) || found_ports.contains(&8443) {
            protocols.push("https".to_string());
        }

        if protocols.is_empty() {
            return None;
        }

        // Try to identify brand from HTTP response
        let brand = identify_brand_by_http(ip).await;
        let onvif_url = if found_ports.contains(&80) {
            Some(format!("http://{}:80/onvif/device_service", ip))
        } else if found_ports.contains(&8080) {
            Some(format!("http://{}:8080/onvif/device_service", ip))
        } else {
            None
        };

        let http_url = if found_ports.contains(&80) {
            Some(format!("http://{}", ip))
        } else if found_ports.contains(&8080) {
            Some(format!("http://{}:8080", ip))
        } else {
            None
        };

        Some(DiscoveredDevice {
            ip: IpAddr::V4(ip),
            mac: None,
            brand,
            model: None,
            name: None,
            protocols,
            rtsp_url,
            onvif_url,
            http_url,
        })
    }
}

/// Send ONVIF WS-Discovery probe via UDP multicast
async fn onvif_discover() -> Vec<DiscoveredDevice> {
    use tokio::net::UdpSocket;

    let probe = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
            xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing"
            xmlns:d="http://schemas.xmlsoap.org/ws/2005/04/discovery"
            xmlns:dn="http://www.onvif.org/ver10/network/wsdl">
  <s:Header>
    <a:Action s:mustUnderstand="1">http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</a:Action>
    <a:MessageID>uuid:__MSG_ID__</a:MessageID>
    <a:ReplyTo><a:Address>http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous</a:Address></a:ReplyTo>
    <a:To s:mustUnderstand="1">urn:schemas-xmlsoap-org:ws:2005:04:discovery</a:To>
  </s:Header>
  <s:Body>
    <d:Probe>
      <d:Types>dn:NetworkVideoTransmitter</d:Types>
    </d:Probe>
  </s:Body>
</s:Envelope>"#;

    let msg_id = uuid::Uuid::new_v4().to_string();
    let probe = probe.replace("__MSG_ID__", &msg_id);

    let mut devices = Vec::new();

    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "Failed to create UDP socket for ONVIF discovery");
            return devices;
        }
    };

    // Send to multicast address
    if let Err(e) = socket.send_to(probe.as_bytes(), "239.255.255.250:3702").await {
        warn!(error = %e, "Failed to send ONVIF probe");
        return devices;
    }

    // Listen for responses for 5 seconds
    let mut buf = vec![0u8; 65535];
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);

    loop {
        match tokio::time::timeout_at(deadline, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, addr))) => {
                let response = String::from_utf8_lossy(&buf[..len]);
                debug!(from = %addr, "ONVIF discovery response");

                // Parse XAddrs from response to get device service URL
                if let Some(xaddrs) = extract_xaddrs(&response) {
                    for xaddr in xaddrs {
                        let ip = addr.ip();
                        devices.push(DiscoveredDevice {
                            ip,
                            mac: None,
                            brand: None,
                            model: None,
                            name: Some(format!("ONVIF Camera {}", ip)),
                            protocols: vec!["onvif".to_string(), "rtsp".to_string()],
                            rtsp_url: None,
                            onvif_url: Some(xaddr),
                            http_url: Some(format!("http://{}", ip)),
                        });
                    }
                }
            }
            Ok(Err(_)) | Err(_) => break,
        }
    }

    info!(count = devices.len(), "ONVIF WS-Discovery found devices");
    devices
}

/// Extract XAddrs URLs from WS-Discovery ProbeMatch response
fn extract_xaddrs(xml: &str) -> Option<Vec<String>> {
    // Simple regex-free XML extraction
    let start_tag = "XAddrs>";
    let start = xml.find(start_tag)? + start_tag.len();
    let end = xml[start..].find('<')? + start;
    let addrs = &xml[start..end];

    Some(addrs.split_whitespace().map(String::from).collect())
}

/// Try to identify camera brand by making HTTP request
async fn identify_brand_by_http(ip: Ipv4Addr) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .danger_accept_invalid_certs(true)
        .build()
        .ok()?;

    // Try common HTTP endpoints
    for port in [80, 8080] {
        let url = format!("http://{}:{}/", ip, port);
        if let Ok(resp) = client.get(&url).send().await {
            let headers = resp.headers().clone();
            let body = resp.text().await.unwrap_or_default().to_lowercase();

            // Check Server header
            if let Some(server) = headers.get("server").and_then(|v| v.to_str().ok()) {
                let server_lower = server.to_lowercase();
                if server_lower.contains("hikvision") || server_lower.contains("hikka") {
                    return Some("Hikvision".into());
                }
                if server_lower.contains("dahua") || server_lower.contains("dss") {
                    return Some("Dahua".into());
                }
                if server_lower.contains("v360") || server_lower.contains("ipc") {
                    return Some("V360 Pro".into());
                }
                if server_lower.contains("reolink") {
                    return Some("Reolink".into());
                }
            }

            // Check response body
            if body.contains("v360") || body.contains("xm530") || body.contains("juan") {
                return Some("V360 Pro".into());
            }
            if body.contains("hikvision") {
                return Some("Hikvision".into());
            }
            if body.contains("dahua") {
                return Some("Dahua".into());
            }
            if body.contains("reolink") {
                return Some("Reolink".into());
            }
            if body.contains("tapo") || body.contains("tp-link") {
                return Some("TP-Link".into());
            }
        }
    }

    None
}

/// Lookup brand from MAC OUI prefix
pub fn brand_from_mac(mac: &str) -> Option<String> {
    let mac_upper = mac.to_uppercase();
    let prefix = if mac_upper.len() >= 8 {
        &mac_upper[..8]
    } else {
        return None;
    };

    for (oui, brand) in CAMERA_OUI {
        if prefix.starts_with(&oui.to_uppercase()) {
            return Some(brand.to_string());
        }
    }

    None
}

#[async_trait]
impl DeviceDiscovery for NetworkScanner {
    async fn scan(&self, request: &ScanRequest) -> Result<Vec<DiscoveredDevice>, DomainError> {
        let mut all_devices = Vec::new();
        let scan_timeout = Duration::from_secs(request.timeout_secs);

        info!(subnets = ?request.subnets, timeout = request.timeout_secs, "Starting network scan");

        for subnet in &request.subnets {
            let ips =
                Self::parse_cidr(subnet).map_err(|e| DomainError::Validation(e))?;

            info!(subnet = %subnet, hosts = ips.len(), "Scanning subnet");

            // Scan in batches of 50 to avoid overwhelming the network
            for chunk in ips.chunks(50) {
                let mut handles = Vec::new();

                for &ip in chunk {
                    let timeout_dur = scan_timeout;
                    handles.push(tokio::spawn(async move {
                        Self::probe_host(ip, timeout_dur).await
                    }));
                }

                for handle in handles {
                    if let Ok(Some(device)) = handle.await {
                        info!(
                            ip = %device.ip,
                            brand = ?device.brand,
                            protocols = ?device.protocols,
                            "Device discovered"
                        );
                        all_devices.push(device);
                    }
                }
            }
        }

        // Run ONVIF WS-Discovery in parallel with port scanning
        if request.include_onvif {
            let onvif_devices = onvif_discover().await;
            for device in onvif_devices {
                // Only add if not already found by port scanning
                if !all_devices.iter().any(|d| d.ip == device.ip) {
                    all_devices.push(device);
                }
            }
        }

        info!(total = all_devices.len(), "Network scan complete");
        Ok(all_devices)
    }

    async fn scan_progress(&self) -> Result<f32, DomainError> {
        // TODO: Track actual progress
        Ok(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cidr_24() {
        let ips = NetworkScanner::parse_cidr("192.168.1.0/24").unwrap();
        assert_eq!(ips.len(), 254);
        assert_eq!(ips[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(ips[253], Ipv4Addr::new(192, 168, 1, 254));
    }

    #[test]
    fn test_parse_cidr_28() {
        let ips = NetworkScanner::parse_cidr("10.0.0.0/28").unwrap();
        assert_eq!(ips.len(), 14);
    }

    #[test]
    fn test_brand_from_mac() {
        assert_eq!(
            brand_from_mac("48:02:2A:11:22:33"),
            Some("Hikvision".into())
        );
        assert_eq!(
            brand_from_mac("A4:14:37:AA:BB:CC"),
            Some("Dahua".into())
        );
        assert_eq!(
            brand_from_mac("00:12:41:11:22:33"),
            Some("V360 Pro".into())
        );
        assert_eq!(brand_from_mac("FF:FF:FF:FF:FF:FF"), None);
    }

    #[test]
    fn test_invalid_cidr() {
        assert!(NetworkScanner::parse_cidr("invalid").is_err());
        assert!(NetworkScanner::parse_cidr("192.168.1.0/33").is_err());
    }
}

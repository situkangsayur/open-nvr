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
    ("54:EF:33", "V360/XiongMai"),
    ("30:7B:C9", "V360/XiongMai"),
    ("48:02:2A", "Hikvision"),
    ("C0:56:E3", "Hikvision"),
    ("28:57:BE", "Hikvision"),
    ("80:48:A5", "Hikvision"),
    ("44:19:B6", "Hikvision"),
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
    ("D0:A4:6F", "Imou/Dahua"),
];

/// Common camera ports to probe
const CAMERA_PORTS: &[u16] = &[
    554,   // RTSP standard
    8554,  // RTSP alternative
    80,    // HTTP/ONVIF
    8080,  // HTTP alternative
    443,   // HTTPS
    8443,  // HTTPS alternative
    3702,  // ONVIF WS-Discovery
    34567, // XiongMai/V360 Pro proprietary
    34568, // XiongMai/V360 Pro control
    9527,  // XiongMai HTTP API
    5000,  // Some Chinese cameras
    8899,  // Yi Camera
    6666,  // Some cheap cameras
    37777, // Dahua
    8000,  // Hikvision
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
        let start = network + 1;
        let end = broadcast.min(network + 1024);

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

        // Probe all camera ports concurrently
        let mut handles = Vec::new();
        for &port in CAMERA_PORTS {
            let addr = SocketAddr::new(IpAddr::V4(ip), port);
            handles.push(tokio::spawn(async move {
                match timeout(Duration::from_millis(800), TcpStream::connect(addr)).await {
                    Ok(Ok(_)) => Some(port),
                    _ => None,
                }
            }));
        }

        for handle in handles {
            if let Ok(Some(port)) = handle.await {
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
            let rtsp_port = if found_ports.contains(&554) { 554 } else { 8554 };
            rtsp_url = Some(format!("rtsp://{}:{}/stream1", ip, rtsp_port));
        }
        if found_ports.contains(&80) || found_ports.contains(&8080) || found_ports.contains(&9527) {
            protocols.push("http".to_string());
        }
        if found_ports.contains(&3702) {
            protocols.push("onvif".to_string());
        }
        if found_ports.contains(&443) || found_ports.contains(&8443) {
            protocols.push("https".to_string());
        }
        if found_ports.contains(&34567) || found_ports.contains(&34568) {
            protocols.push("xmeye".to_string());
            // XiongMai/V360 cameras often have RTSP at 554 but may also use 34567
            if rtsp_url.is_none() {
                rtsp_url = Some(format!("rtsp://{}:554/user=admin&password=&channel=1&stream=0.sdp", ip));
            }
        }
        if found_ports.contains(&37777) {
            protocols.push("dahua".to_string());
        }
        if found_ports.contains(&8000) {
            protocols.push("hikvision".to_string());
        }

        if protocols.is_empty() {
            return None;
        }

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
        } else if found_ports.contains(&9527) {
            Some(format!("http://{}:9527", ip))
        } else {
            None
        };

        Some(DiscoveredDevice {
            ip: IpAddr::V4(ip),
            mac: None,
            brand,
            model: None,
            name: Some(format!("Device {}", ip)),
            protocols,
            rtsp_url,
            onvif_url,
            http_url,
        })
    }

    /// Read ARP table to find devices with their MAC addresses
    async fn arp_scan() -> Vec<DiscoveredDevice> {
        let mut devices = Vec::new();

        // Read /proc/net/arp on Linux
        let arp_content = match tokio::fs::read_to_string("/proc/net/arp").await {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, "Failed to read ARP table");
                return devices;
            }
        };

        for line in arp_content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let ip_str = parts[0];
                let mac = parts[3];

                // Skip incomplete entries
                if mac == "00:00:00:00:00:00" || mac.contains("incomplete") {
                    continue;
                }

                let ip: Ipv4Addr = match ip_str.parse() {
                    Ok(ip) => ip,
                    Err(_) => continue,
                };

                let brand = brand_from_mac(mac);

                devices.push(DiscoveredDevice {
                    ip: IpAddr::V4(ip),
                    mac: Some(mac.to_string()),
                    brand: brand.clone(),
                    model: None,
                    name: brand.map(|b| format!("{} ({})", b, ip_str)),
                    protocols: Vec::new(),
                    rtsp_url: None,
                    onvif_url: None,
                    http_url: None,
                });
            }
        }

        info!(count = devices.len(), "ARP table scan found devices");
        devices
    }

    /// Auto-detect local subnets from network interfaces
    pub async fn detect_local_subnets() -> Vec<String> {
        let mut subnets = Vec::new();

        let output = match tokio::process::Command::new("ip")
            .args(["-4", "addr", "show"])
            .output()
            .await
        {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return subnets,
        };

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("inet ") {
                // Parse: inet 192.168.1.3/24 brd ...
                if let Some(cidr) = line.split_whitespace().nth(1) {
                    // Skip loopback and docker networks
                    if cidr.starts_with("127.")
                        || cidr.starts_with("172.17.")
                        || cidr.starts_with("172.18.")
                        || cidr.starts_with("172.19.")
                        || cidr.starts_with("172.2")
                        || cidr.starts_with("172.3")
                    {
                        continue;
                    }
                    // Convert host address to network address
                    if let Some(slash) = cidr.find('/') {
                        let ip_part = &cidr[..slash];
                        let prefix = &cidr[slash + 1..];
                        if let Ok(ip) = ip_part.parse::<Ipv4Addr>() {
                            if let Ok(pfx) = prefix.parse::<u32>() {
                                let ip_u32 = u32::from(ip);
                                let mask = if pfx == 0 { 0 } else { !((1u32 << (32 - pfx)) - 1) };
                                let network = Ipv4Addr::from(ip_u32 & mask);
                                let subnet = format!("{}/{}", network, pfx);
                                if !subnets.contains(&subnet) {
                                    subnets.push(subnet);
                                }
                            }
                        }
                    }
                }
            }
        }

        info!(subnets = ?subnets, "Detected local subnets");
        subnets
    }
}

/// ONVIF WS-Discovery probe
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

    if let Err(e) = socket.send_to(probe.as_bytes(), "239.255.255.250:3702").await {
        warn!(error = %e, "Failed to send ONVIF probe");
        return devices;
    }

    let mut buf = vec![0u8; 65535];
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

    loop {
        match tokio::time::timeout_at(deadline, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, addr))) => {
                let response = String::from_utf8_lossy(&buf[..len]);
                debug!(from = %addr, "ONVIF discovery response");

                if let Some(xaddrs) = extract_xaddrs(&response) {
                    for xaddr in xaddrs {
                        devices.push(DiscoveredDevice {
                            ip: addr.ip(),
                            mac: None,
                            brand: None,
                            model: None,
                            name: Some(format!("ONVIF Camera {}", addr.ip())),
                            protocols: vec!["onvif".to_string(), "rtsp".to_string()],
                            rtsp_url: None,
                            onvif_url: Some(xaddr),
                            http_url: Some(format!("http://{}", addr.ip())),
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

fn extract_xaddrs(xml: &str) -> Option<Vec<String>> {
    let start_tag = "XAddrs>";
    let start = xml.find(start_tag)? + start_tag.len();
    let end = xml[start..].find('<')? + start;
    let addrs = &xml[start..end];
    Some(addrs.split_whitespace().map(String::from).collect())
}

/// Try to identify camera brand from HTTP response
async fn identify_brand_by_http(ip: Ipv4Addr) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .danger_accept_invalid_certs(true)
        .build()
        .ok()?;

    for port in [80, 8080, 9527] {
        let url = format!("http://{}:{}/", ip, port);
        if let Ok(resp) = client.get(&url).send().await {
            let headers = resp.headers().clone();
            let body = resp.text().await.unwrap_or_default().to_lowercase();

            if let Some(server) = headers.get("server").and_then(|v| v.to_str().ok()) {
                let s = server.to_lowercase();
                if s.contains("hikvision") || s.contains("hikka") { return Some("Hikvision".into()); }
                if s.contains("dahua") || s.contains("dss") { return Some("Dahua".into()); }
                if s.contains("v360") || s.contains("ipc") || s.contains("xmhttp") { return Some("V360/XiongMai".into()); }
                if s.contains("reolink") { return Some("Reolink".into()); }
                if s.contains("thttpd") || s.contains("boa") { return Some("IP Camera".into()); }
            }

            if body.contains("v360") || body.contains("xm530") || body.contains("juan") || body.contains("xmeye") { return Some("V360/XiongMai".into()); }
            if body.contains("hikvision") { return Some("Hikvision".into()); }
            if body.contains("dahua") { return Some("Dahua".into()); }
            if body.contains("reolink") { return Some("Reolink".into()); }
            if body.contains("tapo") || body.contains("tp-link") { return Some("TP-Link".into()); }
        }
    }

    None
}

/// Lookup brand from MAC OUI prefix
pub fn brand_from_mac(mac: &str) -> Option<String> {
    let mac_upper = mac.to_uppercase();
    let prefix = if mac_upper.len() >= 8 { &mac_upper[..8] } else { return None; };
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

        // If no subnets specified, auto-detect from local interfaces
        let subnets = if request.subnets.is_empty() {
            Self::detect_local_subnets().await
        } else {
            request.subnets.clone()
        };

        info!(subnets = ?subnets, timeout = request.timeout_secs, "Starting network scan");

        // Step 1: ARP table scan (instant, finds all recently-seen devices with MACs)
        if request.include_arp {
            let arp_devices = Self::arp_scan().await;
            for device in arp_devices {
                if !all_devices.iter().any(|d: &DiscoveredDevice| d.ip == device.ip) {
                    all_devices.push(device);
                }
            }
        }

        // Step 2: Port scan per subnet
        for subnet in &subnets {
            let ips = Self::parse_cidr(subnet).map_err(|e| DomainError::Validation(e))?;
            info!(subnet = %subnet, hosts = ips.len(), "Scanning subnet");

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
                        // Merge with existing ARP entry if found
                        if let Some(existing) = all_devices.iter_mut().find(|d| d.ip == device.ip) {
                            existing.protocols = device.protocols;
                            existing.rtsp_url = device.rtsp_url;
                            existing.onvif_url = device.onvif_url;
                            existing.http_url = device.http_url;
                            if device.brand.is_some() {
                                existing.brand = device.brand;
                            }
                            if device.name.is_some() {
                                existing.name = device.name;
                            }
                        } else {
                            all_devices.push(device);
                        }
                    }
                }
            }
        }

        // Step 3: ONVIF WS-Discovery
        if request.include_onvif {
            let onvif_devices = onvif_discover().await;
            for device in onvif_devices {
                if let Some(existing) = all_devices.iter_mut().find(|d| d.ip == device.ip) {
                    if !existing.protocols.contains(&"onvif".to_string()) {
                        existing.protocols.push("onvif".to_string());
                    }
                    if existing.onvif_url.is_none() {
                        existing.onvif_url = device.onvif_url;
                    }
                } else {
                    all_devices.push(device);
                }
            }
        }

        info!(total = all_devices.len(), "Network scan complete");
        Ok(all_devices)
    }

    async fn scan_progress(&self) -> Result<f32, DomainError> {
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
        assert_eq!(brand_from_mac("48:02:2A:11:22:33"), Some("Hikvision".into()));
        assert_eq!(brand_from_mac("A4:14:37:AA:BB:CC"), Some("Dahua".into()));
        assert_eq!(brand_from_mac("00:12:41:11:22:33"), Some("V360 Pro".into()));
        assert_eq!(brand_from_mac("54:EF:33:45:28:95"), Some("V360/XiongMai".into()));
        assert_eq!(brand_from_mac("FF:FF:FF:FF:FF:FF"), None);
    }

    #[test]
    fn test_invalid_cidr() {
        assert!(NetworkScanner::parse_cidr("invalid").is_err());
        assert!(NetworkScanner::parse_cidr("192.168.1.0/33").is_err());
    }
}

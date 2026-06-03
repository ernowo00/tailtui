use serde_json::Value;
use std::net::Ipv4Addr;

use crate::domain::error::AppError;
use crate::domain::model::{Machine, TailscaleStatus};

pub fn parse_status_json(raw: &str) -> Result<TailscaleStatus, AppError> {
    let value: Value =
        serde_json::from_str(raw).map_err(|e| AppError::ParseFailed(e.to_string()))?;

    let backend_state = value
        .get("BackendState")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let self_node = value.get("Self").and_then(Value::as_object);
    let self_name = self_node
        .and_then(|node| node.get("HostName"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            self_node
                .and_then(|node| node.get("DNSName"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });

    let tailnet_name = value
        .get("CurrentTailnet")
        .and_then(Value::as_object)
        .and_then(|t| t.get("Name"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let ips = self_node
        .and_then(|node| node.get("TailscaleIPs"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let exit_node = value
        .get("ExitNodeStatus")
        .and_then(Value::as_object)
        .and_then(|status| status.get("Online"))
        .and_then(Value::as_bool)
        .and_then(|online| {
            if online {
                value
                    .get("ExitNodeStatus")
                    .and_then(Value::as_object)
                    .and_then(|status| status.get("ID"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            } else {
                None
            }
        });

    let magic_suffix = value
        .get("CurrentTailnet")
        .and_then(|t| t.get("MagicDNSSuffix"))
        .and_then(Value::as_str)
        .or_else(|| value.get("MagicDNSSuffix").and_then(Value::as_str));

    let machines = value
        .get("Peer")
        .and_then(Value::as_object)
        .map(|peers| {
            let mut nodes = peers
                .values()
                .filter_map(|p| parse_machine(p, magic_suffix))
                .collect::<Vec<Machine>>();
            nodes.sort_by(|a, b| match b.online.cmp(&a.online) {
                std::cmp::Ordering::Equal => a.sort_key().cmp(b.sort_key()),
                other => other,
            });
            nodes
        })
        .unwrap_or_default();

    Ok(TailscaleStatus {
        backend_state,
        tailnet_name,
        self_name,
        ips,
        exit_node,
        machines,
    })
}

fn parse_machine(peer: &Value, magic_suffix: Option<&str>) -> Option<Machine> {
    let obj = peer.as_object()?;
    let hostname = obj
        .get("HostName")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let nickname_from_json = ["ComputerNickname", "MachineNickname", "Nickname"]
        .iter()
        .find_map(|k| obj.get(*k).and_then(Value::as_str))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let nickname_from_dns = obj.get("DNSName").and_then(Value::as_str).and_then(|dns| {
        magic_suffix
            .filter(|s| !s.is_empty())
            .map(|suffix| trim_tailscale_dns_suffix(dns, suffix))
            .filter(|s| !s.is_empty())
    });

    let nickname = nickname_from_json.or(nickname_from_dns);

    let online = obj.get("Online").and_then(Value::as_bool).unwrap_or(false);
    let ipv4 = obj
        .get("TailscaleIPs")
        .and_then(Value::as_array)
        .and_then(|ips| ips.iter().filter_map(Value::as_str).find_map(extract_ipv4));
    Some(Machine {
        nickname,
        hostname,
        ipv4,
        online,
    })
}

/// Mirrors `tailscale.com/util/dnsname`.TrimSuffix for MagicDNS labels.
fn dns_has_suffix(name: &str, suffix: &str) -> bool {
    let name = name.trim_end_matches('.');
    let suffix = suffix.trim_matches('.');
    if suffix.is_empty() {
        return false;
    }
    let name_base = name.trim_end_matches(suffix);
    name_base.len() < name.len() && name_base.ends_with('.')
}

fn trim_tailscale_dns_suffix(name: &str, suffix: &str) -> String {
    let trimmed = if dns_has_suffix(name, suffix) {
        let name = name.trim_end_matches('.');
        let suffix = suffix.trim_matches('.');
        name.trim_end_matches(suffix).trim_end_matches('.')
    } else {
        name.trim_end_matches('.')
    };
    trimmed.to_string()
}

fn extract_ipv4(raw: &str) -> Option<String> {
    let candidate = raw.split('/').next().unwrap_or(raw);
    match candidate.parse::<Ipv4Addr>() {
        Ok(ip) => Some(ip.to_string()),
        Err(_) => None,
    }
}

/// First IPv4 in `Self.TailscaleIPs`-style strings (skips IPv6, supports `/mask`).
pub fn first_ipv4_from_addrs(ips: &[String]) -> Option<String> {
    ips.iter().find_map(|s| extract_ipv4(s))
}

#[cfg(test)]
mod tests {
    use super::{first_ipv4_from_addrs, parse_status_json, trim_tailscale_dns_suffix};

    #[test]
    fn parses_peer_machine_and_extracts_ipv4() {
        let raw = r#"
        {
          "BackendState": "Running",
          "CurrentTailnet": { "MagicDNSSuffix": "tail-scale.ts.net" },
          "Peer": {
            "peer1": {
              "HostName": "db-host",
              "DNSName": "db-node.alice.tail-scale.ts.net.",
              "Online": true,
              "TailscaleIPs": ["fd7a:115c:a1e0::1234", "100.88.77.66/32"]
            }
          }
        }
        "#;
        let parsed = parse_status_json(raw).expect("status json should parse");
        assert_eq!(parsed.machines.len(), 1);
        let machine = &parsed.machines[0];
        assert_eq!(machine.hostname, "db-host");
        assert_eq!(machine.nickname.as_deref(), Some("db-node.alice"));
        assert!(machine.online);
        assert_eq!(machine.ipv4.as_deref(), Some("100.88.77.66"));
    }

    #[test]
    fn trims_magicdns_suffix_like_tailscale() {
        assert_eq!(
            trim_tailscale_dns_suffix("laptop.alice.tail-scale.ts.net.", "tail-scale.ts.net"),
            "laptop.alice"
        );
    }

    #[test]
    fn first_ipv4_skips_ipv6() {
        let ips = vec![
            "fd7a:115c:a1e0::1".to_string(),
            "100.99.1.2/32".to_string(),
        ];
        assert_eq!(first_ipv4_from_addrs(&ips).as_deref(), Some("100.99.1.2"));
    }
}

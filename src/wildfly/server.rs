use anyhow::bail;
use std::fmt::Display;

// ------------------------------------------------------ server group

/// WildFly domain server group assignment.
#[derive(Clone, Debug, PartialEq)]
pub enum ServerGroup {
    MainServerGroup,
    OtherServerGroup,
}

impl ServerGroup {
    pub fn parse_group(input: &str) -> Option<ServerGroup> {
        if input.eq_ignore_ascii_case("msg") || input.eq_ignore_ascii_case("main-server-group") {
            Some(ServerGroup::MainServerGroup)
        } else if input.eq_ignore_ascii_case("osg")
            || input.eq_ignore_ascii_case("other-server-group")
        {
            Some(ServerGroup::OtherServerGroup)
        } else {
            None
        }
    }
}

impl Display for ServerGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerGroup::MainServerGroup => write!(f, "main-server-group"),
            ServerGroup::OtherServerGroup => write!(f, "other-server-group"),
        }
    }
}

// ------------------------------------------------------ server

/// A managed server definition within a WildFly domain host.
#[derive(Clone, Debug, PartialEq)]
pub struct Server {
    pub name: String,
    pub server_group: ServerGroup,
    pub offset: u16,
    pub autostart: bool,
}

impl Server {
    pub fn parse_servers(input: &str) -> anyhow::Result<Vec<Server>> {
        input.split(',').map(Server::parse_server).collect()
    }

    pub fn parse_server(input: &str) -> anyhow::Result<Server> {
        let parts: Vec<&str> = input.split(':').collect();
        if parts.is_empty() {
            bail!("Invalid input format");
        }

        let name = parts[0].to_string();
        if name.is_empty() {
            bail!("Invalid input format");
        }

        let mut server_group = ServerGroup::MainServerGroup;
        let mut offset: u16 = 0;
        let mut autostart = false;
        let mut remaining = &parts[1..];

        // Try to consume server group
        if let Some((&first, rest)) = remaining.split_first() {
            if let Some(sg) = ServerGroup::parse_group(first) {
                server_group = sg;
                remaining = rest;
            } else if !first.eq_ignore_ascii_case("start") && first.parse::<u16>().is_err() {
                bail!("Invalid server group: '{}'", first);
            }
        }

        // Try to consume offset
        if let Some((&first, rest)) = remaining.split_first() {
            if let Ok(o) = first.parse::<u16>() {
                offset = o;
                remaining = rest;
            } else if !first.eq_ignore_ascii_case("start") {
                bail!("Invalid input format");
            }
        }

        // Try to consume "start"
        if let Some((&first, rest)) = remaining.split_first() {
            if first.eq_ignore_ascii_case("start") {
                autostart = true;
                remaining = rest;
            } else {
                bail!("Invalid input format");
            }
        }

        if !remaining.is_empty() {
            bail!("Invalid input format");
        }

        Ok(Server {
            name,
            server_group,
            offset,
            autostart,
        })
    }

    pub fn with_offset(&self, offset: u16) -> Server {
        Server {
            name: self.name.clone(),
            server_group: self.server_group.clone(),
            offset,
            autostart: self.autostart,
        }
    }

    pub fn add_server_op(&self, host: &str) -> String {
        if self.offset > 0 {
            format!(
                "/host={}/server-config={}:add(group={},socket-binding-port-offset={},auto-start={})",
                host, self.name, self.server_group, self.offset, self.autostart
            )
        } else {
            format!(
                "/host={}/server-config={}:add(group={},auto-start={})",
                host, self.name, self.server_group, self.autostart
            )
        }
    }
}

// ------------------------------------------------------ server offsets

pub const DEFAULT_SERVER_OFFSET: u16 = 100;

pub fn apply_offsets(servers: Vec<Server>, offset: u16) -> Vec<Server> {
    if servers.len() > 1 {
        let mut last_offset = 0;
        servers
            .iter()
            .enumerate()
            .map(|(index, server)| {
                if index == 0 {
                    server.clone()
                } else {
                    let server_with_offset = if server.offset == 0 {
                        server.with_offset(last_offset + offset)
                    } else {
                        server.clone()
                    };
                    last_offset = server_with_offset.offset;
                    server_with_offset
                }
            })
            .collect::<Vec<_>>()
    } else {
        servers
    }
}

// ------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wildfly::ServerGroup::{MainServerGroup, OtherServerGroup};

    // ------------------------------------------------------ parse server tests

    #[test]
    fn parse_server_name_only() {
        let result = Server::parse_server("server1").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_server_group() {
        let result = Server::parse_server("server1:msg").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);

        let result = Server::parse_server("server1:main-server-group").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);

        let result = Server::parse_server("server1:osg").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, OtherServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);

        let result = Server::parse_server("server1:other-server-group").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, OtherServerGroup);
        assert_eq!(result.offset, 0);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_offset() {
        let result = Server::parse_server("server1:123").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_autostart() {
        let result = Server::parse_server("server1:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_name_server_group_offset() {
        let result = Server::parse_server("server1:msg:123").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(!result.autostart);
    }

    #[test]
    fn parse_server_name_server_group_autostart() {
        let result = Server::parse_server("server1:msg:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 0);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_name_offset_autostart() {
        let result = Server::parse_server("server1:123:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_name_server_group_offset_autostart() {
        let result = Server::parse_server("server1:msg:123:start").unwrap();
        assert_eq!(result.name, "server1");
        assert_eq!(result.server_group, MainServerGroup);
        assert_eq!(result.offset, 123);
        assert!(result.autostart);
    }

    #[test]
    fn parse_server_invalid() {
        let result = Server::parse_server("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_server_invalid_server_group() {
        let result = Server::parse_server("server1:groupA");
        assert!(result.is_err());
    }

    #[test]
    fn parse_server_invalid_offset() {
        let result = Server::parse_server("server1:msg:abc:start");
        assert!(result.is_err());
    }

    #[test]
    fn parse_server_offset_before_group() {
        let result = Server::parse_server("server1:123:groupA:start");
        assert!(result.is_err());
    }

    #[test]
    fn add_server_op_no_offset() {
        let server = Server {
            name: "server-one".to_string(),
            server_group: MainServerGroup,
            offset: 0,
            autostart: true,
        };
        assert_eq!(
            server.add_server_op("primary"),
            "/host=primary/server-config=server-one:add(group=main-server-group,auto-start=true)"
        );
    }

    #[test]
    fn add_server_op_with_offset() {
        let server = Server {
            name: "server-two".to_string(),
            server_group: OtherServerGroup,
            offset: 100,
            autostart: false,
        };
        assert_eq!(
            server.add_server_op("secondary"),
            "/host=secondary/server-config=server-two:add(group=other-server-group,socket-binding-port-offset=100,auto-start=false)"
        );
    }

    // ------------------------------------------------------ apply offsets tests

    #[test]
    fn apply_offsets_empty() {
        let servers = vec![];
        assert_eq!(apply_offsets(servers, 100), vec![]);
    }

    #[test]
    fn apply_offsets_single_server() {
        let server = Server::parse_server("server1").unwrap();
        let input = vec![server.clone()];
        let expected = vec![server.clone()];
        assert_eq!(apply_offsets(input, 100), expected);
    }

    #[test]
    fn apply_offsets_multiple_servers() {
        let server0 = Server::parse_server("server0").unwrap();
        let server1 = Server::parse_server("server1").unwrap();
        let server2 = Server::parse_server("server2").unwrap();
        let server3 = Server::parse_server("server3").unwrap();
        let input = vec![
            server0.clone(),
            server1.clone(),
            server2.clone(),
            server3.clone(),
        ];
        let expected = vec![
            server0.clone(),
            server1.with_offset(100),
            server2.with_offset(200),
            server3.with_offset(300),
        ];
        assert_eq!(apply_offsets(input, 100), expected);
    }

    #[test]
    fn apply_offsets_multiple_servers_custom_offset() {
        let server0 = Server::parse_server("server0").unwrap();
        let server1 = Server::parse_server("server1").unwrap();
        let server2 = Server::parse_server("server2:50").unwrap();
        let server3 = Server::parse_server("server3").unwrap();
        let input = vec![
            server0.clone(),
            server1.clone(),
            server2.clone(),
            server3.clone(),
        ];
        let expected = vec![
            server0.clone(),
            server1.with_offset(100),
            server2.with_offset(50),
            server3.with_offset(150),
        ];
        assert_eq!(apply_offsets(input, 100), expected);
    }
}

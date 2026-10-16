use crate::models::network_stack::NetworkStack;
use crate::models::nic::NicType;
use crate::models::nodes::router::Router;
use crate::models::operating_system::OperatingSystem;
use crate::models::routing_stack::RoutingStack;
use crate::utils::os_commands::utils::format_command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use crate::models::protocol::RoutingProtocol;
use crate::models::routes::route_config::RouteConfig;

#[derive(Debug, Clone)]
pub struct OsCommand {
    pub expect: String,
    pub send: SendType,
    pub can_fail: bool,
}


#[derive(Debug, Clone, Default)]
pub enum SendType {
    #[default]
    NewLine,
    Text(String, bool),
    Ctrl(char),
    Wait(u64)
}

impl OsCommand {
    pub fn new_text<E: AsRef<str>, S: AsRef<str>>(expect: E, send: S, new_line: bool, can_fail: bool) -> OsCommand {
        OsCommand {
            expect: expect.as_ref().to_string(),
            send: SendType::Text(send.as_ref().to_string(), new_line),
            can_fail,
        }
    }

    #[allow(unused)]
    pub fn new_control<E: AsRef<str>>(expect: E, send: char, can_fail: bool) -> OsCommand {
        OsCommand {
            expect: expect.as_ref().to_string(),
            send: SendType::Ctrl(send),
            can_fail,
        }
    }

    pub fn new_line<E: AsRef<str>>(expect: E) -> OsCommand {
        OsCommand {
            expect: expect.as_ref().to_string(),
            send: SendType::NewLine,
            can_fail: false,
        }
    }
}

impl Display for SendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SendType::NewLine => String::new(),
            SendType::Text(text, _) => text.to_owned(),
            SendType::Ctrl(char) => format!("Ctrl-{}", char),
            SendType::Wait(time) => format!("Wait {} ms", time),
        };
        write!(f, "{}", str)
    }
}


#[derive(Debug,Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeserializedOsCommandType {
    Simple(String),
    Other(DeserializedOsCommand)
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct DeserializedOsCommand {
    pub expect: Option<String>,
    #[serde(flatten)]
    pub send: DeserializedSendType,
    #[serde(default)]
    pub can_fail: bool,
    #[serde(default)]
    pub assert_true: Vec<(String, String)>
}

#[derive(Debug,Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeserializedSendType {
    Text {
        send: String
    },
    Ctrl {
        control: char,
    },
    Wait {
        wait: u64
    }
}

pub struct CommandContext<'a> {
    pub router_name: &'a str,
    pub router: &'a Router,
    pub os: &'a OperatingSystem,
    pub network_stack: Option<&'a NetworkStack>,
    pub routing_stack: Option<&'a RoutingStack>,
}

impl DeserializedOsCommandType {
    pub fn to_os_command(&self, context: &CommandContext, to_replace: Option<&HashMap<String, String>>) -> Option<OsCommand> {
        match self {
            DeserializedOsCommandType::Simple(send) => {
                let send = match to_replace {
                    None => send.to_owned(),
                    Some(to_replace) => format_command(send, to_replace)
                };

                Some(OsCommand::new_text(&context.os.input_ready, send, true, false))
            },
            DeserializedOsCommandType::Other(command) => {
                for (key, value) in &command.assert_true {
                    let mut context_map = context.to_key_value();

                    if let Some(to_replace) = to_replace {
                        for (to_replace_key, to_replace_value) in to_replace {
                            // Removes the { } in the key
                            let adjusted_key = &to_replace_key[1..to_replace_key.len()-1];

                            if !context_map.contains_key(adjusted_key) {
                                context_map.insert(adjusted_key, to_replace_value.to_owned());
                            }
                        }
                    }

                    if let Some(value_to_check) = context_map.get(key.as_str()) {
                        if value_to_check != value {
                            return None;
                        }
                    }
                }

                let send = match &command.send {
                    DeserializedSendType::Text { send } => {
                        let send = match to_replace {
                            None => send.to_owned(),
                            Some(to_replace) => format_command(send, to_replace)
                        };

                        SendType::Text(send, true)
                    }
                    DeserializedSendType::Ctrl { control: char } => SendType::Ctrl(*char),
                    DeserializedSendType::Wait { wait } => SendType::Wait(*wait)
                };

                let expect = match &command.expect {
                    Some(expect) => expect.to_owned(),
                    None => context.os.input_ready.to_owned()
                };

                Some(OsCommand {
                    expect,
                    send,
                    can_fail: command.can_fail,
                })
            }
        }
    }
}

impl CommandContext<'_> {
    pub fn to_replace_map(&self, nic_type: Option<&NicType>) -> anyhow::Result<HashMap<String, String>> {
        let mut old_map = self.to_key_value();
        let mut map = HashMap::new();

        for (key, value) in old_map.drain() {
            map.insert(format!("{{{key}}}"), value);
        }

        if let Some(nic_type) = nic_type {
            map.insert(String::from("{INTERFACE_PREFIX}"), self.os.interface_prefix(&nic_type)?);
        }

        Ok(map)
    }

    fn to_key_value(&self) -> HashMap<&str, String> {
        let mut map = HashMap::new();

        // OperatingSystem
        map.insert("OS", self.router.os_name.clone());
        map.insert("INPUT_READY", self.os.input_ready.clone());

        if let Some(login) = &self.os.login {
            map.insert("LOGIN", login.clone());
        }

        if let Some(password) = &self.os.password {
            map.insert("PASSWORD", password.clone());
        }

        map.insert("NETWORK_STACK", self.os.network_stack.clone());

        if let Some(routing_stack) = &self.os.routing_stack {
            map.insert("ROUTING_STACK", routing_stack.clone());
        }

        map.insert("INTERFACES_START_AT", self.os.interfaces_start_at.to_string());
        map.insert("GAP_BETWEEN_INTERFACES", self.os.gap_between_interfaces.to_string());

        map.insert("ROUTER_ID", self.router.id.clone());

        let mut routing_protocol_static = false;
        let mut routing_protocol_rip = false;
        let mut routing_protocol_ospf = false;
        let mut routing_protocol_bgp = false;
        let mut routing_protocol_mpls = false;
        let mut routing_protocol_mixed = false;

        match &self.router.routes_config {
            RouteConfig::Static(_) => routing_protocol_static = true,
            RouteConfig::Rip(_) => routing_protocol_rip = true,
            RouteConfig::Ospf(_) => routing_protocol_ospf = true,
            RouteConfig::Bgp(_) => routing_protocol_bgp = true,
            RouteConfig::Mpls => routing_protocol_mpls = true,
            RouteConfig::Mixed(mixed) => {
                routing_protocol_mixed = true;

                for routing_protocol in mixed.keys() {
                    match routing_protocol {
                        RoutingProtocol::Static => routing_protocol_static = true,
                        RoutingProtocol::Rip => routing_protocol_rip = true,
                        RoutingProtocol::Ospf => routing_protocol_ospf = true,
                        RoutingProtocol::Bgp => routing_protocol_bgp = true,
                        RoutingProtocol::Mpls => routing_protocol_mpls = true,
                        RoutingProtocol::Mixed => panic!("Mixed protocol recursion"),
                    };
                }
            }
        }

        map.insert("ROUTING_PROTOCOL_STATIC", routing_protocol_static.to_string());
        map.insert("ROUTING_PROTOCOL_RIP", routing_protocol_rip.to_string());
        map.insert("ROUTING_PROTOCOL_OSPF", routing_protocol_ospf.to_string());
        map.insert("ROUTING_PROTOCOL_BGP", routing_protocol_bgp.to_string());
        map.insert("ROUTING_PROTOCOL_MPLS", routing_protocol_mpls.to_string());
        map.insert("ROUTING_PROTOCOL_MIXED", routing_protocol_mixed.to_string());

        map
    }
}
use std::collections::HashMap;
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use crate::models::operating_system::OperatingSystem;
use crate::utils::os_commands::utils::format_command;

#[derive(Clone)]
pub struct OsCommand {
    pub expect: String,
    pub send: SendType,
    pub can_fail: bool,
}


#[derive(Clone, Default)]
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
            SendType::Text(text, _) => text.to_owned(),
            SendType::Ctrl(char) => format!("Ctrl-{}", char),
            SendType::NewLine => String::new(),
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
    pub can_fail: bool
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

impl DeserializedOsCommandType {
    pub fn to_os_command(&self, os: &OperatingSystem, to_replace: Option<&HashMap<&str, String>>) -> OsCommand {
        match self {
            DeserializedOsCommandType::Simple(send) => {
                let send = match to_replace {
                    None => send.to_owned(),
                    Some(to_replace) => format_command(send, to_replace)
                };

                OsCommand::new_text(&os.input_ready, send, true, false)
            },
            DeserializedOsCommandType::Other(command) => {
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
                    None => os.input_ready.to_owned()
                };

                OsCommand {
                    expect,
                    send,
                    can_fail: command.can_fail,
                }
            }
        }
    }
}

use std::sync::mpsc;
use clap::{App, Arg};
use env_logger::Builder;
use log::LevelFilter;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct BroadcasterVal(u32, messages::Val);

use messaging_udp::messages;
pub fn reliable_broadcast_bb(send_msg: &mpsc::Sender<messages::Message>, recv_msg: &mut mpsc::Receiver<messages::Message>, honest: bool, my_id: u32, num_parties: u32, t_s: u32, sender: bool) -> messages::Val {
    let mut val = messages::Val::Lambda;
    if honest {
        let mut echo_messages: HashMap<BroadcasterVal, HashSet<u32>> = HashMap::new();
        let mut ready_messages: HashMap<BroadcasterVal, HashSet<u32>> = HashMap::new();
        let mut ready_message_sent = false;
        
        if sender {
            info!("Running tardigrade reliable broadcast SENDER protocol");
            val = messages::Val::One;
            let msg = messages::Message::ProposeBB {
                v: val,
                broadcaster: my_id,
            };
            messages::send_message(msg, send_msg);
            let mut set = HashSet::new();
            set.insert(my_id);
            echo_messages.insert(BroadcasterVal(my_id, val), set);
            let msg = messages::Message::EchoBB {
                v: val,
                broadcaster: my_id,
                echoer: my_id,
            };
            messages::send_message(msg, send_msg);
        }
        
        info!("Running tardigrade reliable broadcast RECEIVER protocol");
        
        loop {
            let msg = recv_msg.recv().unwrap();
            info!("received message: {:?}", msg);
            match msg {
                messages::Message::ProposeBB { v, broadcaster } => {
                    match echo_messages.get_mut(&BroadcasterVal(broadcaster, v)) {
                        Some(_) => continue,
                        _ => {
                            let mut set: HashSet<u32> = HashSet::new();
                            set.insert(my_id);
                            echo_messages.insert(BroadcasterVal(broadcaster, v), set);
                            let msg = messages::Message::EchoBB {
                                v,
                                broadcaster,
                                echoer: my_id,
                            };
                            messages::send_message(msg, send_msg);
                        },
                    };

                },
                messages::Message::EchoBB { v, broadcaster, echoer } => {
                    match echo_messages.get_mut(&BroadcasterVal(broadcaster, v)) {
                        Some(set) => {
                            if set.insert(echoer) {

                                if set.len() >= ((num_parties - t_s) as usize) && !ready_message_sent {
                                    ready_message_sent = true;
                                    match ready_messages.get_mut(&BroadcasterVal(broadcaster, v)) {
                                        Some(set) => {
                                            if set.insert(my_id) {
                                                // need to check the case count for ready messages
                                                // for our triggers
                                                if set.len() >= ((num_parties - t_s) as usize) {
                                                    val = v;
                                                    break;
                                                }
                                            }
                                        },
                                        _ => {
                                            let mut set: HashSet<u32> = HashSet::new();
                                            set.insert(my_id);
                                            ready_messages.insert(BroadcasterVal(broadcaster, v), set);
                                        },
                                    };
                                    let msg = messages::Message::ReadyBB {
                                        v,
                                        broadcaster,
                                        readier: my_id,
                                    };
                                    messages::send_message(msg, send_msg);
                                }
                            }
                        },
                        _ => {
                            let mut set: HashSet<u32> = HashSet::new();
                            set.insert(echoer);
                            echo_messages.insert(BroadcasterVal(broadcaster, v), set);
                        },
                    };
                },
                messages::Message::ReadyBB { v, broadcaster, readier } => {
                    match ready_messages.get_mut(&BroadcasterVal(broadcaster, v)) {
                        Some(set) => {
                            if set.insert(readier) {

                                if set.len() >= ((t_s + 1) as usize) && !ready_message_sent {
                                    ready_message_sent = true;
                                    set.insert(my_id);
                                    let msg = messages::Message::ReadyBB {
                                        v,
                                        broadcaster,
                                        readier: my_id,
                                    };
                                    messages::send_message(msg, send_msg);
                                }

                                if set.len() >= ((num_parties - t_s) as usize) {
                                    val = v;
                                    break;
                                }
                            }
                        },
                        _ => {
                            let mut set: HashSet<u32> = HashSet::new();
                            set.insert(readier);
                            ready_messages.insert(BroadcasterVal(broadcaster, v), set);
                        },
                    }
                },
                _ => continue,
            }
        }
    }
    val
}

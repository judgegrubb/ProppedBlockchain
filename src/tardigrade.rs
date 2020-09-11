
use std::sync::mpsc;
use clap::{App, Arg};
use env_logger::Builder;
use log::LevelFilter;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct BroadcasterVal(u32, messages::Val);

use messaging_udp::messages;
pub fn reliable_broadcast_bb(send_msg: &mpsc::Sender<messages::Message>, recv_msg: &mut mpsc::Receiver<messages::Message>, honest: bool, my_id: u32) {
    if honest {
        let mut val = messages::Val::Lambda;
        let mut messages: HashMap<BroadcasterVal, &mut HashSet<u32>> = HashMap::new();
        loop {
            let msg = recv_msg.recv().unwrap();
            info!("received message: {:?}", msg);
            match msg {
                messages::Message::ProposeBB { v, broadcaster } => {
                    match messages.get(&BroadcasterVal(broadcaster, v)) {
                        Some(_) => continue,
                        _ => {
                            let set: &mut HashSet<u32> = &mut HashSet::new();
                            set.insert(broadcaster);
                            messages.insert(BroadcasterVal(broadcaster, v), &mut set);
                            let msg = messages::Message::EchoBB {
                                v,
                                broadcaster,
                                echoer: my_id,
                            };
                            messages::send_message(msg, send_msg);
                        },
                    };

                },
                messages::Message::EchoBB { v, broadcaster, echoer } => continue,
                messages::Message::ReadyBB { v, broadcaster, readier } => continue,
                _ => continue,
            }
        }
    }
}

pub fn reliable_broadcast_bb_sender(send_msg: &mpsc::Sender<messages::Message>, recv_msg: &mut mpsc::Receiver<messages::Message>, honest: bool, my_id: u32) {
    if honest {
    
    }
}

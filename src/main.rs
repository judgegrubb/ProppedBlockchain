#[macro_use] 
extern crate log;
extern crate env_logger;
extern crate messaging_udp;
extern crate clap;

use chrono::Local;
use clap::{Arg, App};
use env_logger::Builder;
use log::LevelFilter;
use messaging_udp::messages;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::*;
use std::sync::mpsc;
use std::time::Duration;
use std::thread;

#[derive(Debug)]
enum Protocol {
    Async,
    Control,
    Propped,
    Thunderella,
}

fn send_prepare(round: u32, val: messages::Val, send_msg: &mpsc::Sender<messages::Message>)  {
    let msg = messages::Message::Prepare {
        k: round,
        b: val,
    };
    messages::send_message(msg, send_msg);
}

fn send_propose(round: u32, val: messages::Val, send_msg: &mpsc::Sender<messages::Message>)  {
    let msg = messages::Message::Propose {
        k: round,
        b: val,
    };
    messages::send_message(msg, send_msg);
}

fn propose(send_msg: &mpsc::Sender<messages::Message>, recv_msg: &mut mpsc::Receiver<messages::Message>, val: messages::Val, round: u32, n: u32, t_s: u32) -> HashSet<messages::Val> {
    info!("proposing {:?}", val);
    let mut vals = HashSet::new();
    let mut prepare_sent = HashSet::new();
    let mut prepare_count = HashMap::new();
    let mut propose_count = HashMap::new();
    let mut prop = HashSet::new();

    prepare_sent.insert(val);
    prepare_count.insert(val, 1);

    send_prepare(round, val, &send_msg);
    
    loop {
        let msg = recv_msg.recv().unwrap();
        info!("received message: {:?}", msg);
        match msg {
            messages::Message::Prepare { k, b } => {
                if k != round {
                    continue;
                }
                let count = prepare_count.entry(b).or_insert(1);
                *count += 1;
                if *count == t_s + 1 && !prepare_sent.contains(&b) {
                    prepare_sent.insert(b);
                    send_prepare(k, b, &send_msg);
                }
                if *count >= n - t_s && !vals.contains(&b) {
                    let mut prop_count = 0;
                    if vals.is_empty() {
                        send_propose(k, b, &send_msg);
                        prop_count = match propose_count.get(&b) {
                            Some(j) => j + 1,
                            None => 1,
                        };
                        propose_count.insert(b, prop_count);
                    }
                    vals.insert(b);
                    if prop_count >= n - t_s {
                        prop.insert(b);
                        break;
                    }
                }
            },
            messages::Message::Propose { k, b } => {
                if k != round {
                    continue;
                }
                let count = match propose_count.get(&b) {
                    Some(j) => j + 1,
                    None => 1,
                };
                propose_count.insert(b, count);
                if count >= n - t_s && vals.contains(&b) {
                    prop.insert(b);
                    break;
                }
            },
            _ => continue,
        };
    }

    info!("values agreed {:?}", prop);
    prop
}

fn graded_consensus(send_msg: &mpsc::Sender<messages::Message>, recv_msg: &mut mpsc::Receiver<messages::Message>, val: messages::Val, round: u32, n: u32, t_s: u32) -> (messages::Val, u32) {
    info!("gc input: {:?}", val);
    let b_1 = val;
    let prop_1 = propose(send_msg, recv_msg, b_1, round, n, t_s);
    let b_2 = if prop_1.len() == 1 && prop_1.contains(&b_1) {
        b_1
    } else {
        messages::Val::Lambda
    };
    let prop_2 = propose(send_msg, recv_msg, b_2, round+1, n, t_s);
    if prop_2.len() == 2 {
        let comp_one: HashSet<messages::Val> = vec![messages::Val::One, messages::Val::Lambda].into_iter().collect();
        let comp_zero: HashSet<messages::Val> = vec![messages::Val::Zero, messages::Val::Lambda].into_iter().collect();
        if prop_2.symmetric_difference(&comp_one).next().is_none() {
            (messages::Val::One, 1)
        } else if prop_2.symmetric_difference(&comp_zero).next().is_none() {
            (messages::Val::Zero, 1)
        } else {
            (messages::Val::Lambda, 0)
        }
    } else if prop_2.len() == 1 {
        match prop_2.iter().next().unwrap() {
            messages::Val::Zero => (messages::Val::Zero, 2),
            messages::Val::One => (messages::Val::One, 2),
            messages::Val::Lambda => (messages::Val::Lambda, 0),
        }
    } else {
        (messages::Val::Lambda, 0)
    }
}

//fn async_byzantine_agreement(send_msg: &mpsc::Sender<String>, recv_msg: &mut mpsc::Receiver<String>, val: messages::Val, round: u32, n: u32, t_s: u32) -> (messages::Val, u32) {
//}

fn parse_sleep_command(command: &str) -> Result<Vec<String>, Self::Err> {

}

fn parse_wake_command(command: &str) -> Result<Vec<String>, Self::Err> {
}

fn confirmation_thread(send_msg: &mpsc::Sender<messages::Message>, confirmation_time: i32) {
    let mut round = 1;
    loop {
        let sleep_time = time::Duration::from_secs(confirmation_time);
        thread::sleep(sleep_time);
        
        let msg = messages::Message::Prepare {
            block: round,
            b: messages::Val::Zero,
        };
        messages::send_message(msg, send_msg);
        
        round = round + 1;
    }
}

fn run_async_agreement(mut messaging_obj: messages::MessagingObjects) {
}

//TODO catch latency messages before putting them into channel
//TODO set up keeping track of how much latency for each other person in LAN
//TODO implement latency on specific messages based on src address
fn run_control_server(mut messaging_obj: messages::MessagingObjects, confirmation_time: i32) {
    // TODO spin off coinflip mechanism
    // This is the only piece that we care about incoming messages, right?

    // TODO Need confirm timer to send out messages every so often
    let confirmation_send_msg = messaging_obj.send_msg.clone();
    thread::spawn(move || {
        confirmation_thread(confirmation_send_msg, confirmation_time);
    });

    // loop over user input
    // based on commands, send out messages
    loop {
        let mut input = String::new();
        io::stdin::().read_line(&mut input).expect("error: unable to read user input");


    }
}

fn run_propped_agreement(mut messaging_obj: messages::MessagingObjects) {
}


fn run_thunderella_agreement(mut messaging_obj: messages::MessagingObjects) {
}

fn run_reliable_broadcast(mut messaging_obj: messages::MessagingObjects) {
    
}

fn main() {
    // flags to handle what exact protocol we want to run
    let matches = App::new("Propped Blockchain Simulation")
        .version("0.1")
        .author("Elijah Grubb <egrubb@umd.edu>")
        .about("Simulating various optimistic protocols on top of a blockchain")
        .arg(Arg::with_name("protocol")
             .short("p")
             .long("protocol")
             .value_name("PROTOCOL")
             .help("Sets which protocol to run. CONTROL, THUNDER, ASYNC, PROP, AGNOST")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("confirmation")
             .short("ct")
             .long("confirmation")
             .value_name("CONFIRMATION")
             .help("Sets the length of epochs for the underlying blockchain confirmation in seconds. Only the control server uses this parameter. Default is 10 seconds")
             .takes_value(true))
        .get_matches();
    
    // format our logger.
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let mut messaging_obj = messages::setup_udp();

    let active_protocol = match matches.value_of("protocol").unwrap_or("CONTROL") {
        "ASYNC" => Protocol::Async,
        "THUNDER" => Protocol::Thunderella,
        "PROP" => Protocol::Propped, 
        "CONTROL" | _ => Protocol::Control,
    };
    info!("Running {:?} protocol.", active_protocol);

    let confirmation_time = matches.value_of("confirmation").unwrap_or("10").parse::<i32>().unwrap();

    match active_protocol {
        Protocol::Async => run_async_agreement(messaging_obj),
        Protocol::Control => run_control_server(messaging_obj, confirmation_time),
        Protocol::Propped => run_propped_agreement(messaging_obj),
        Protocol::Thunderella => run_thunderella_agreement(messaging_obj),
    };

//    let t_s = messaging_obj.n / 2;
//
//    info!("waiting 5 seconds to simulate synchronous network");
//
//    thread::sleep(Duration::from_secs(5));
//
//    info!("starting propose test for {} parties with a sec param of {}", messaging_obj.n, t_s); 
//
//    for i in 0..10 {
//        let mut rng = thread_rng();
//        let x = rng.gen_range(0, 2);
//        let val = match x {
//            0 => messages::Val::Zero,
//            1 => messages::Val::One,
//            _ => messages::Val::Lambda,
//        };
//        let (bit, grade) = graded_consensus(&messaging_obj.send_msg, &mut messaging_obj.recv_msg, val, i*2, messaging_obj.n, t_s);
//        println!("ROUND {}: proposed: {:?}, agreed: ({},{})", i, val, bit, grade);
//    }
    
    // spin down
    // shut down threads?
    // send msg to term_send and term_rcv
    // join threads
    // shut down socket?
    info!("Exiting program");
}


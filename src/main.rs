#[macro_use] 


extern crate log;
extern crate env_logger;
extern crate messaging_udp;
extern crate clap;

mod tardigrade;

use chrono::Local;
use clap::{App, Arg};
use env_logger::Builder;
use log::LevelFilter;
use messaging_udp::messages;
//use rand::prelude::*;
use std::collections::{HashMap, HashSet};
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
    Tardigrade,
}

//fn async_byzantine_agreement(send_msg: &mpsc::Sender<String>, recv_msg: &mut mpsc::Receiver<String>, val: messages::Val, round: u32, n: u32, t_s: u32) -> (messages::Val, u32) {
//}

//fn parse_sleep_command(command: &str) -> Result<Vec<String>, Self::Err> {
//}

//fn parse_wake_command(command: &str) -> Result<Vec<String>, Self::Err> {
//}

fn confirmation_thread(send_msg: &mpsc::Sender<messages::Message>, confirmation_time: u64) {
    let mut round = 1;
    loop {
        let sleep_time = Duration::from_secs(confirmation_time);
        thread::sleep(sleep_time);
        
        let msg = messages::Message::Prepare {
            k: round,
            b: messages::Val::Zero,
        };
        messages::send_message(msg, send_msg);
        
        round = round + 1;
    }
}

//fn run_async_agreement(messaging_obj: messages::MessagingObjects) {
//}

//TODO catch latency messages before putting them into channel
//TODO set up keeping track of how much latency for each other person in LAN
//TODO implement latency on specific messages based on src address
fn run_control_server(messaging_obj: messages::MessagingObjects, confirmation_time: u64) {
    // TODO spin off coinflip mechanism
    // This is the only piece that we care about incoming messages, right?

    // TODO Need confirm timer to send out messages every so often
    let confirmation_send_msg = messaging_obj.send_msg.clone();
    thread::spawn(move || {
        confirmation_thread(&confirmation_send_msg, confirmation_time);
    });

    // loop over user input
    // based on commands, send out messages
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("error: unable to read user input");


    }
}

//fn run_propped_agreement(messaging_obj: messages::MessagingObjects) {
//}


//fn run_thunderella_agreement(messaging_obj: messages::MessagingObjects) {
//}

fn run_tardigrade(mut messaging_obj: messages::MessagingObjects, kappa: u32, delta: u32, payload_size: u32, batch_size: u32) {
    info!("running tardigrade SMR protocol");

    tardigrade::smr(&messaging_obj.send_msg, &mut messaging_obj.recv_msg, messaging_obj.my_id, messaging_obj.n, messaging_obj.t_s, messaging_obj.t_a, payload_size, batch_size, kappa, delta);
    
    
    
    
    //let val = match messaging_obj.my_name.as_str() {
        //"node0" => tardigrade::reliable_broadcast_bb(&messaging_obj.send_msg, &mut messaging_obj.recv_msg, true, messaging_obj.my_id, messaging_obj.n, messaging_obj.t_s, true), 
        //_ => tardigrade::reliable_broadcast_bb(&messaging_obj.send_msg, &mut messaging_obj.recv_msg, true, messaging_obj.my_id, messaging_obj.n, messaging_obj.t_s, false),
    //};
    //info!("Reliable Broadcast Output: {:?}", val);
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
             .help("Sets which protocol to run. CONTROL, THUNDER, ASYNC, PROP, TARDI")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("confirmation")
             .short("ct")
             .long("confirmation")
             .value_name("CONFIRMATION")
             .help("Sets the length of epochs for the underlying blockchain confirmation in seconds. Only the control server uses this parameter. Default is 10 seconds")
             .takes_value(true))
        .arg(Arg::with_name("kappa")
             .short("k")
             .long("kappa")
             .value_name("KAPPA")
             .help("set the kappa security param for synchronous protocols")
             .takes_value(true))
        .arg(Arg::with_name("delta")
             .short("del")
             .long("delta")
             .value_name("DELTA")
             .help("set the network bound in a synchronous protocol in seconds")
             .takes_value(true))
        .arg(Arg::with_name("batch")
             .short("bat")
             .long("batch")
             .value_name("BATCH")
             .help("set the batch size for our blockchain protocol")
             .takes_value(true))
        .arg(Arg::with_name("payload")
             .short("pay")
             .long("payload")
             .value_name("PAYLOAD")
             .help("size of the payloads in tardigrade")
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
        "TARDI" => Protocol::Tardigrade,
        "CONTROL" | _ => Protocol::Control,
    };
    info!("Running {:?} protocol.", active_protocol);

    let confirmation_time = matches.value_of("confirmation").unwrap_or("10").parse::<u64>().unwrap();
    let kappa = matches.value_of("kappa").unwrap_or("5").parse::<u32>().unwrap();
    let delta = matches.value_of("delta").unwrap_or("50").parse::<u32>().unwrap();
    let batch_size = matches.value_of("batch").unwrap_or("400").parse::<u32>().unwrap();
    let payload_size = matches.value_of("payload").unwrap_or("0").parse::<u32>().unwrap();

    match active_protocol {
        //Protocol::Async => run_async_agreement(messaging_obj),
        //Protocol::Propped => run_propped_agreement(messaging_obj),
        //Protocol::Thunderella => run_thunderella_agreement(messaging_obj),
        Protocol::Tardigrade => run_tardigrade(messaging_obj, kappa, delta, payload_size, batch_size),
        _ => run_control_server(messaging_obj, confirmation_time),
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


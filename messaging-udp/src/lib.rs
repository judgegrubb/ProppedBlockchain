#[macro_use] 
extern crate log;
extern crate env_logger;

pub mod messages {

use serde::{Serialize, Deserialize};
use std::{fs, thread};
use std::net::UdpSocket;
use std::process::Command;
use std::sync::mpsc;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum Val {
    Zero,
    One,
    Lambda,
}

//TODO add a timestamp t to Prepare, Propose, and Coin for latency adding purposes
//TODO also add to prepare implementation and parse_message
//TODO add a user tag so we know who we're receiving from
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    Prepare { k: u32, b: Val },
    Propose { k: u32, b: Val },
    Coin { k: u32 },
    Latency { delay: u32, parties: u32 },
    Ping,
    Sleep { name: String },
    Confirm { block: u32, b: Val },
    Wake { name : String },
    EchoBB { v: Val, broadcaster: u32, echoer: u32},
    ReadyBB { v: Val, broadcaster: u32, readier: u32},
    ProposeBB { v: Val, broadcaster: u32 },
}

impl Message {
    fn prepare(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

fn parse_message(message: String) -> Message {
    serde_json::from_str(&message).unwrap()
}

fn sender_thread_controller(socket: &UdpSocket, rx: mpsc::Receiver<Message>, _term: mpsc::Receiver<String>, send_addrs: Vec<String>) {
    //TODO implement termination check to kill thread at some point
    
    info!("IPs to send to: {:?}", send_addrs);
    for received in rx {
        info!("sending message: {:?}", received);
        let received = received.prepare();
        for addr in &send_addrs {
            let message = received.clone();
            let _result = socket.send_to(&message.into_bytes(), addr).expect("failed to send message");
            //info!("{} sent message with {} bytes", addr, result);
        }
    }
}

fn receiver_thread_controller(socket: &UdpSocket, tx: mpsc::Sender<Message>, _term: mpsc::Receiver<String>) {
    //TODO implement termination check to kill thread at some point
    //TODO returning who a message is sent from
    loop {
        let mut buf: [u8; 50] = [0; 50];
        let (num_of_bytes, _src_addr) = socket.recv_from(&mut buf).expect("no data received");
        //info!("received message of size {} from {:?}", num_of_bytes, src_addr);
        let filled_buf = &mut buf[..num_of_bytes];
        let message = String::from_utf8(filled_buf.to_vec()).unwrap();
        //info!("message received: {}", message);
        tx.send(parse_message(message)).unwrap();
    }
}

// Some ugly code for parsing the /etc/hosts file
// in order to get a list of all the ips for all the
// parties on our LAN
fn get_ips_from_etc_hosts(my_ip: String) -> Vec<String> {
    let mut ips: Vec<String> = Vec::new();

    let contents = fs::read_to_string("/etc/hosts")
        .expect("something went wrong reading the file");
    let contents = contents.trim();
    let contents = contents.split('\n');

    for line in contents {
        let ip = match line.split_whitespace().next() {
            Some(text) => {
                format!("{}:8080", text)
            },
            None => {
                panic!("Problem parsing /etc/hosts")
            },
        };
        if ip == "127.0.0.1:8080" || ip == my_ip {
            continue;
        } else {
            ips.push(ip);
        }
    }
    ips
}

fn get_my_party_num() -> u32 {
    let output = Command::new("bash")
                         .arg("-c")
                         .arg("hostname")
                         .output()
                         .expect("failed to execute process");
    //let stdout = output.stdout;
    let stdout: String = String::from_utf8(output.stdout).unwrap();    
    let index = stdout.find('.').unwrap();
    let num: u32 = stdout[index-1..index].parse().unwrap();
    num
}

fn get_my_hostname(my_party_num: u32) -> String {
    let ip = format!("10.10.1.{}:8080", my_party_num);
    ip
}

pub struct MessagingObjects {
    pub my_name: String,
    pub my_id: u32,
    pub send_msg: mpsc::Sender<Message>,
    pub recv_msg: mpsc::Receiver<Message>,
    pub socket: UdpSocket,
    pub send_handle: thread::JoinHandle<()>,
    pub recv_handle: thread::JoinHandle<()>,
    pub send_term: mpsc::Sender<String>,
    pub recv_term: mpsc::Sender<String>,
    pub n: u32,
}

pub fn setup_udp() -> MessagingObjects {
    info!("setup starting");
    let my_party_num = get_my_party_num();
    let my_name = format!("node{}", my_party_num);
    let host_ip = get_my_hostname(my_party_num);
    info!("listening on {}", host_ip);
    let other_ips = get_ips_from_etc_hosts(host_ip.clone());
    let num_of_parties = (other_ips.len() as u32) + 1;
    let socket = UdpSocket::bind(host_ip).expect("couldn't bind to address");
    // need copy of socket for each thread
    let socket_listener = socket.try_clone().expect("couldn't clone socket");
    let socket_sender = socket.try_clone().expect("couldn't clone socket");

    let (listener_tx, listener_rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (sender_tx, sender_rx) = mpsc::channel();
    let (term_send_tx, term_send_rx) = mpsc::channel();
    let (term_rcv_tx, term_rcv_rx) = mpsc::channel();


    let recv_handle = thread::spawn(move || {
        info!("starting receiver thread");
        receiver_thread_controller(&socket_listener, listener_tx, term_rcv_rx);
    });

    let send_handle = thread::spawn(move || {
        info!("starting sender thread");
        sender_thread_controller(&socket_sender, sender_rx, term_send_rx, other_ips);
    });

    info!("setup complete");    

    MessagingObjects {
        my_name,
        my_id: my_party_num,
        send_msg: sender_tx,
        recv_msg: listener_rx,
        socket,
        send_handle,
        recv_handle,
        send_term: term_send_tx,
        recv_term: term_rcv_tx,
        n: num_of_parties,
    }
}

pub fn send_message(msg: Message, send_msg_chan: &mpsc::Sender<Message>) {
    info!("sending message {:?}", msg);
    mpsc::Sender::clone(send_msg_chan).send(msg).unwrap();
}

}

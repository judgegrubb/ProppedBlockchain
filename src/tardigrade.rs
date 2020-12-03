
use std::sync::mpsc;
use clap::{App, Arg};
use env_logger::Builder;
use log::LevelFilter;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use std::thread::sleep;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct BroadcasterVal(u32, messages::Val);

use messaging_udp::messages;

pub fn generate_transactions(payload_size: u32, batch_size: u32, num_parties: u32) -> HashSet<messages::Transaction> {
    return HashSet::new();
}

pub fn sleep_until(time_zero: Instant, correct_time_difference: u32) {
    let time_now = Instant::now();
    let time_since_zero = time_now.duration_since(time_zero);
    if time_since_zero.as_millis < correct_time_difference {
        sleep(Duration::from_millis(correct_time_difference - time_since_zero.as_millis));
    }
}

pub fn smr(send_msg: &mpsc::Sender<messages::Message>, 
                      recv_msg: &mut mpsc::Receiver<messages::Message>, 
                      my_id: u32, 
                      num_parties: u32, 
                      t_s: u32, 
                      t_a: u32,
                      payload_size: u32,
                      batch_size: u32,
                      kappa: u32,
                      delta: u32) {

    let mut epochs: HashMap<u32, u32> = HashMap::new();
    let mut k = 1;

    // for now, we are just going to do a single epoch
    // start at time T_k = (\Delta + \kappa\Delta) * (k - 1)

    // okay first we set up our data structures and collect our input data
    // Epochs_i[k] := 1
    epochs.insert(k, 1);
    // \Beta := \emptyset
    let mut beta: HashSet<messages::Transaction> = HashSet::new();
    // \Sigma := \emptyset
    let mut sigma: HashSet<messages::Transaction> = HashSet::new();
    // buf_i is our input
    // generate batch of inputs given L (batch_size) and n (number of parties)
    let mut buf_i: HashSet<messages::Transaction> = generate_transactions(payload_size, batch_size, num_parties);

    // threshold encrypt buffer C_j
    // multicast C_j

    // then we send our encrypted buffer to everyone and collect everyone else's buffers
    // while |\Sigma| <= t_s + t_a and T < T_k + \Delta
    //      UPON receiving (k, C_i) from P_i
    //          if sig is valid and its the first one from P_i in epoch k
    //              add it to \Beta and add sig to \Sigma

    // At time T_k + \Delta (so we should have everyone's buffers)
    // now we run BLA on (\Beta,\Sigma)
    let mut (new_beta, new_sigma) = bla(send_msg, recv_msg, beta, sigma, kappa, delta, num_parties, my_id);

    // while T < T_k + \Delta + (\kappa*\Delta)
    //      if BLA produces t_s + t_a - valid output
    //          set \Beta*,\Sigma* to it

    // at time T_k + \Delta + (\kappa * \Delta)
    //      if BLA did not produce valid output in time
    //          set (\Beta*,\Sigma*) = (\Beta,\Sigma)
    
    // if BLA output is t_s valid then set (\Beta*,\Sigma*) to it

    // else set (\Beta*,\Sigma*) = (\Beta,\Sigma)

    // now run ACS on \Beta*
    
    // update Block_i[k] with ACS output
    
    // remove entries that were successfully committed from buf_i
}

pub fn bla(send_msg: &mpsc::Sender<messages::Message>, 
           recv_msg: &mut mpsc::Receiver<messages::Message>, 
           beta: HashSet<messages::Transaction>, 
           sigma: HashSet<messages::Transaction>, 
           kappa: u32, 
           delta: u32,
           num_parties: u32,
           my_id: u32) -> (HashSet<messages::Transaction>,HashSet<messages::Transaction>) {
    // k* round number based on security parameter kappa
    let mut k_star = 0;
    // \Beta* transactions
    let mut b_star = beta;
    // \Sigma* transactions but organized (?)
    let mut s_star = sigma;
    // Certificate
    let mut c_star: messages::Certificate = { };
    //let mut c_star = HashSet<
   
    let mut k = 1;
    while k <= kappa { 
    //      at time 5k-5 * \Delta
        let ((b_new, s_new, c_new), grade) = gc(send_msg, recv_msg, k_star, b_star, s_star, c_star, delta, num_parties, my_id);
    //      at time 5k * \Delta 
        if grade > 0 {
            k_star = k;
            b_star = b_new;
            s_star = s_new;
            c_star = c_new;
        }
        if grade == 2 {
            (b_star,s_star)
        }
        k = k + 1;
    }

    (HashSet::new(),HashSet::new())
}

pub fn gc(send_msg: &mpsc::Sender<messages::Message>, 
          recv_msg: &mut mpsc::Receiver<messages::Message>, 
          smrK: u32,
          k_prime: u32,
          beta: HashSet<messages::Transaction>, 
          sigma: HashSet<messages::Transaction>, 
          c_prime: Certificate,
          delta: u32,
          num_parties: u32,
          my_id: u32) -> ((HashSeet<messages::Transaction>,HashSet<messages::Transaction>,Certificate),u32) {
    
    let t = (num_parties + 1) / 2 + ((num_parties + 1) % 2 != 0); // let t = \ceil((n + 1)/2)
    
    // at time 0 run parallel executions of propose for senders P1, ... Pn 
    let time_zero = Instant::now();
    // we're going to assume leader(k) = 1
    let proposer_id = 1;
    // each using input (k',\Beta,\Sigma, C') each with output (\Beta_j, \Sigma_j)
    let (B_l, Sig_l) = propose(send_msg, recv_msg, smrK, proposer_id, k_prime, beta, sigma, c_prime, delta, num_parties, t, my_id);

    // at time 3 * \Delta call leader(k) to obtain the response l
    sleep_until(time_zero, (3 * delta));

    // if (\Beta_l, \Sigma_l) != null send commit(k, \Beta_l, \Sigma_l) to every party
    if B_l.len() > 0 && Sig_l.len() > 0 {
        let msg = TardiGCCommit {
            smrk,
            k: k_prime,
            B: B_l,
            Sig: Sig_l,
            committer_id: my_id,
        };
        messages::send_message(msg, send_msg);
    }

    // at time 4 * \Delta if at least t correctly formed commit messages for l from
    sleep_until(time_zero, (4 * delta));
    // distinct parties have been received then form a k-certificate C for B_l, Sigma_l
    // send notify k B_l Sigma_l C to every party
    // output ((B_l, \Sigma_l, C),2) and terminate
    
    // at time 5 * \Delta if a correctly formed notify message (k, B, Sigma, C)
    // has been received output ((B, \sigma, C), 1) and terminate
    
    // else output ((\bot),0) and terminate
}

pub fn propose(send_msg: &mpsc::Sender<messages::Message>, 
               recv_msg: &mut mpsc::Receiver<messages::Message>, 
               smrK: u32,
               proposer_id: u32,
               k: u32,
               beta: HashSet<messages::Transaction>, 
               sigma: HashSet<messages::Transaction>, 
               cert: messages::Certificate,
               delta: u32,
               num_parties: u32,
               t: u32,
               my_id: u32) -> ((HashSet<messages::Transaction>,HashSet<messages::Transaction>)) {
    // at time 0 send status for my_id = status(k, beta, sigma, cert) to sender_id
    let msg = messages::Message::TardiProposeStatus {
        smrK,
        k,
        B: beta,
        Sig: sigma,
        C: cert,
        sender_id: my_id,
        proposer_id,
    };
    messages::send_message(msg, send_msg);

    let time_zero = Instant::now();
    let mut status_messages: HashSet<messages::Message::TardiProposeStatus> = HashSet::new();
    let mut propose_message = messages::Message::TardiProposePropose {
        smrK,
        k,
        Statuses: HashSet::new(),
        sender_id: my_id,
        proposer_id,
    }

    if (my_id == proposer_id) {
        // at time \Delta if P* has received at least s >= t correctly formed status messages
        sleep(Duration::from_millis(delta));
        loop {
            let msg = recv_msg.try_recv().unwrap();
            let msg = match msg {
                Some(sent_message) => sent_message,
                // else return \bot
                None => return (HashSet::new(),HashSet::new()),
            }
    // status_1, ..., status_t from distinct parties then P* sets
    // m = (propose, status1, ..., status_t) and sends m to all parties
            match msg {
                messages::Message::TardiProposeStatus status_msg => {
                    if status_msg.smrK != smrK {
                        continue;
                    }
                    propose_message.Statuses.insert(status_msg);
                },
                _ => continue,
            }

            if propose_message.Statuses.len() >= t {
                messages::send_message(propose_message, send_msg);
                break;
            }
        }
    
    } else {
        // at time 2 * \Delta if I receive a Propose message m from P*
        // then send m to all parties
        // otherwise output \bot
        sleep(Duration::from_millis(2 * delta));
        loop {
            let msg = recv_msg.try_recv().unwrap();
            let msg = match msg{
                Some(sent_message) => sent_message,
                // otherwise output \bot
                None => return (HashSet::new(), HashSet::new()),
            }

            match msg {
                messages::Message::TardiProposePropose recv_propose_message => {
                    if recv_propose_message.smrK != smrK || recv_propose_message.sender_id != proposer_id {
                        continue;
                    }
                    propose_message = recv_propose_message;
                    break;
                },
                _ => continue,
            }
        }
    }

    // at time 3 * \Delta let m_j be the propose message from party j
    sleep_until(time_zero, (3 * delta));

    // if t.e. j s.t. m_j != m output \bot

    let mut max_k = 0;
    let mut b_prime = HashSet::new();
    let mut sig_prime = HashSet::new();
    // otherwise let status_max = status(k', beta', sigma', cert') be the m
    // with maximum k'
    for status in propose_message.Statuses {
        if status.k > max_k {
            b_prime = status.B;
            sig_prime = status.Sig;
        }
    }
    
    // output (beta', sigma')
    (b_prime, sig_prime)
}

pub fn acs_star(send_msg: &mpsc::Sender<messages::Message>, 
           recv_msg: &mut mpsc::Receiver<messages::Message>, 
           beta: HashSet<messages::Transaction>, 
           kappa: u32, 
           delta: u32,
           t_a: u32,
           t_s: u32,
           num_parties: u32,
           my_id: u32) -> (HashSet<messages::Transaction>,HashSet<messages::Transaction>) {

    // S_j = acs_star(beta)

    // UPON S_j, multicast commit (S_j)
    
    // UPON receiving at least t_s + 1 sigs on commit(S)
    // form a certificate sigma
    // multicast sigma
    // output S
    // terminate
    
    // UPON receiving sigma for S
    // multicast sigma
    // output S
    // terminate


}

pub fn acs_star(send_msg: &mpsc::Sender<messages::Message>, 
           recv_msg: &mut mpsc::Receiver<messages::Message>, 
           beta: HashSet<messages::Transaction>, 
           kappa: u32, 
           delta: u32,
           t_a: u32,
           t_s: u32,
           num_parties: u32,
           my_id: u32) -> (HashSet<messages::Transaction>,HashSet<messages::Transaction>) {

    // let S* = {i : BA_i output 1}
    // let s = |S*|
   
    // boolean conditions:
    // C_1(v): at least n - t_s executions {Bcast_i} have output v
    // C_1: t.e. v for which C_1(v) is true
    // C_2(v): s >= n - t_a, all executions {BA_i} have terminated, majority of {Bcast_i} executions have output v
    // C_2: t.e. v for which C_2(v) is true
    // C_3: s >= n - t_a, all executions {BA_i} have terminated, all executions {Bcast_i} have terminated

    // for all i
    //      run Bcast_i with P_i as the sender, where P_i uses input v_i
    //      when Bcast_i terminates with output v'_i:
    //          if BA_i has not begin:
    //              run BA_i using input 1
    // When s >= n - t_a run any executions of BA_i that have not yet begun
    //      with input 0
    
    // Exit 1: if at any point C_1(v) = true for some v, ouptut {v}
    // Exit 2: if at any point !C_1 && C_2(v) for some v, output {v}
    // Exit 3: if at any point !C_1 && !C_2 && C_3, output S := {v'_i}_{i \in S*}
    

    // after outputting
    //      continue to participate in any ongoing Bcast executions
    //      once C_1 == true stop participating in any ongoing BA executions

}

pub fn bb(send_msg: &mpsc::Sender<messages::Message>,
          recv_msg: &mut mpsc::Receiver<messages::Message>,
          sender_id: u32,
          beta: HashSet<messages::Transaction>, 
          t_s: u32,
          num_parties: u32,
          my_id: u32) -> HashSet<messages::Transaction> {
    
    // if sender send beta to all

    // upon receiving v* from sender send echo(v*) to all parties
    
    // upon receiving echo(v*) on same v* from n - t_s parties:
    //      if ready(v*) has not been sent
    //          send ready(v*) to all parties

    // upon receiving ready(v*) on same v* from t_s + 1 parties:
    //      if ready(v*) has not been sent
    //          send ready(v*) to all parties

    // upon receiving ready(v*) on same v* from n - t_s parties:
    //      output v* and terminate
}


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

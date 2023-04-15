use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use std_semaphore::Semaphore;

use crate::{
    containers::{coffee_container::CoffeContainer, container::Container, resourse::Resourse},
    dispensers::dispenser::Dispenser,
    helpers::{ingredients::Ingredients, ticket::Ticket},
};

const DISPENSERS: i32 = 2;
const INGREDIENTS: [Ingredients; 5] = [
    Ingredients::Coffee,
    Ingredients::Milk,
    Ingredients::Water,
    Ingredients::Foam,
    Ingredients::Cacao,
];

#[derive(Debug)]
pub struct CoffeMachine {
    coffe_made: i32,
    req_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
    res_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
}

impl CoffeMachine {
    pub fn new() -> Self {
        let coffe_made = 0;
        let req_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>> = HashMap::new();
        let res_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>> = HashMap::new();
        Self {
            coffe_made,
            req_monitors,
            res_monitors,
        }
    }

    fn init_containers_monitors(&mut self) {
        for i in INGREDIENTS.iter().copied() {
            match i {
                Ingredients::Coffee => {
                    let coffee_req_monitor =
                        Arc::new((Mutex::new(Resourse::new(0, 0)), Condvar::new()));
                    let coffee_res_monitor =
                        Arc::new((Mutex::new(Resourse::new(0, 0)), Condvar::new()));
                    self.req_monitors.insert(i, coffee_req_monitor);
                    self.res_monitors.insert(i, coffee_res_monitor);
                }
                Ingredients::Milk => {}
                Ingredients::Foam => {}
                Ingredients::Cacao => {}
                Ingredients::Water => {}
                _ => {}
            }
        }
    }

    fn init_containers(&mut self) -> Vec<JoinHandle<()>> {
        let mut containers = Vec::with_capacity(INGREDIENTS.len());

        for i in INGREDIENTS.iter().copied() {
            match i {
                Ingredients::Coffee => {
                    if let Some(req) = self.req_monitors.get(&i) {
                        if let Some(res) = self.res_monitors.get(&i) {
                            let request_monitor = req.clone();
                            let response_monitor = res.clone();
                            containers.push(thread::spawn(move || {
                                let mut coffe_container = CoffeContainer::new();
                                coffe_container.start(request_monitor, response_monitor);
                            }));
                        }
                    }
                }
                Ingredients::CoffeGrain => {}
                Ingredients::Milk => {}
                Ingredients::Foam => {}
                Ingredients::Cacao => {}
                Ingredients::Water => {}
            }
        }

        containers
    }

    fn init_dispensers(
        &self,
        machine_sem: Arc<Semaphore>,
        order_lock: Arc<Mutex<VecDeque<Ticket>>>,
    ) -> Vec<JoinHandle<()>> {
        let mut dispensers = Vec::with_capacity(DISPENSERS as usize);

        for i in 0..DISPENSERS {
            let req_monitors = self.req_monitors.clone();
            let res_monitors = self.res_monitors.clone();
            let order_monitor = order_lock.clone();
            let m_monitor = machine_sem.clone();

            dispensers.push(thread::spawn(move || {
                println!("creating dispenser {}", i);
                let dispenser = Dispenser::new(i);
                dispenser.start(m_monitor, order_monitor, &req_monitors, &res_monitors);
            }));
        }

        dispensers
    }

    fn notify_new_ticket(
        &self,
        lock: &Arc<Mutex<VecDeque<Ticket>>>,
        sem: &Arc<Semaphore>,
        mut new_ticket: Ticket,
    ) -> Result<(), String> {
        if let Ok(mut ticket_vec) = lock.lock() {
            new_ticket.ready_to_read();
            ticket_vec.push_front(new_ticket);
            sem.release();
            return Ok(());
        };
        Err("[error] - ticket monitor failed".to_string())
    }

    //TODO: make reader
    fn read_ticket(&self, i: i32) -> Option<Ticket> {
        if i == 3 {
            return Some(Ticket::new(-1));
        }
        Some(Ticket::new(i * 10))
    }

    fn kill_dispensers(&self, dispensers: Vec<JoinHandle<()>>) {
        for d in dispensers {
            if d.join().is_ok() {
                println!("[global]  - dispenser killed")
            };
        }
    }

    pub fn start(&mut self) {
        let machine_sem = Arc::new(Semaphore::new(0));
        let mut q = VecDeque::new();
        let ticket_lock = Arc::new(Mutex::new(q));
        self.init_containers_monitors();
        let _containers = self.init_containers();
        let dispensers = self.init_dispensers(machine_sem.clone(), ticket_lock.clone());

        for i in 1..4 {
            // TODO: read new line
            // create ticket
            match self.read_ticket(i) {
                Some(ticket) => match self.notify_new_ticket(&ticket_lock, &machine_sem, ticket) {
                    Ok(_) => {
                        println!("[coffe machine] - Coffe Machine send {} new ticket", i)
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                },
                None => {
                    println!("[coffe machine] - Coffe Machine finished.");
                    break;
                }
            }
        }
        println!("[coffe machine] - Coffe Machine finished.");
        self.kill_dispensers(dispensers);
    }
}

impl Default for CoffeMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod coffemachine_test {
    use std::{
        collections::VecDeque,
        sync::{Arc, Condvar, Mutex},
    };

    use std_semaphore::Semaphore;

    use crate::{
        coffee_machine::{CoffeMachine, DISPENSERS},
        helpers::ticket::Ticket,
    };

    #[test]
    fn it_should_initialize_with_0_coffe_made() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        assert_eq!(coffemachine.coffe_made, 0);
    }

    #[test]
    fn it_should_signal_coffe_dispenser() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let ticket = Ticket::new(0);
        let new_ticket = Ticket::new(10);
        let mut q = VecDeque::new();
        q.push_front(ticket);
        let ticket_mut = Arc::new(Mutex::new(q));
        let sem = Arc::new(Semaphore::new(0));

        let _ = coffemachine
            .notify_new_ticket(&ticket_mut.clone(), &sem, new_ticket)
            .unwrap();

        if let Ok(guard) = ticket_mut.lock() {
            assert!(ticket.is_not_ready())
        };
    }

    #[test]
    fn it_should_initilize_dispensers() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let new_ticket = Ticket::new(10);

        let ticket_mut = Arc::new(Mutex::new(VecDeque::new()));
        let sem = Arc::new(Semaphore::new(0));

        let dispensers = coffemachine.init_dispensers(sem, ticket_mut);
        assert_eq!(dispensers.len(), DISPENSERS as usize)
    }
}

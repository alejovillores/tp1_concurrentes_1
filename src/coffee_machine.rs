use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    containers::{coffee_container::CoffeContainer, container::Container, resourse::Resourse},
    dispensers::dispenser::Dispenser,
    helpers::{ingredients::Ingredients, ticket::Ticket},
};

const DISPENSERS: i32 = 1;
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
        machine_monitor: Arc<(Mutex<bool>, Condvar)>,
        ticket_monitor: Arc<(Mutex<Ticket>, Condvar)>,
    ) -> Vec<JoinHandle<()>> {
        let mut dispensers = Vec::with_capacity(DISPENSERS as usize);

        let req_monitors = self.req_monitors.clone();
        let res_monitors = self.res_monitors.clone();

        dispensers.push(thread::spawn(move || {
            let coffe_dispenser = Dispenser::new(0);
            coffe_dispenser.start(
                machine_monitor,
                ticket_monitor,
                &req_monitors,
                &res_monitors,
            );
        }));

        dispensers
    }

    fn finished(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<bool, String> {
        if let Ok(guard) = lock.lock() {
            // As long as the value inside the `Mutex<bool>` is `true`, we wait
            if let Ok(mut _guard) = cvar.wait_while(guard, |status| *status) {
                *_guard = true;
                return Ok(true);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    fn notify_new_ticket(
        &self,
        lock: &Mutex<Ticket>,
        cvar: &Condvar,
        new_ticket: Ticket,
    ) -> Result<(), String> {
        if let Ok(mut ticket) = lock.lock() {
            *ticket = new_ticket;
            ticket.ready_to_read();
            cvar.notify_all();
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
        let machine_monitor = Arc::new((Mutex::new(false), Condvar::new()));
        let ticket_monitor = Arc::new((Mutex::new(Ticket::new(0)), Condvar::new()));
        self.init_containers_monitors();
        let _containers = self.init_containers();
        let dispensers = self.init_dispensers(machine_monitor.clone(), ticket_monitor.clone());

        for i in 1..4 {
            let (lock, cvar) = &*machine_monitor;
            match self.finished(lock, cvar) {
                Ok(_) => {
                    println!("[coffe machine] - Coffe Machine is free for new tickets");
                    self.coffe_made += 1;
                }
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
            // TODO: read new line
            // create ticket
            match self.read_ticket(i) {
                Some(ticket) => {
                    let (lock_ticket, cvar_ticket) = &*ticket_monitor;
                    match self.notify_new_ticket(lock_ticket, cvar_ticket, ticket) {
                        Ok(_) => {
                            println!("[coffe machine] - Coffe Machine send {} new ticket", i)
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            break;
                        }
                    }
                }
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
    use std::sync::{Arc, Condvar, Mutex};

    use crate::{coffee_machine::CoffeMachine, helpers::ticket::Ticket};

    #[test]
    fn it_should_initialize_with_0_coffe_made() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        assert_eq!(coffemachine.coffe_made, 0);
    }

    #[test]
    fn it_should_be_true_when_dispensers_are_free() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let monitor = Arc::new((Mutex::new(false), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffemachine.finished(lock, cvar) {
            Ok(result) => assert!(result),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_signal_coffe_dispenser() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let ticket = Ticket::new(0);
        let new_ticket = Ticket::new(10);

        let monitor = Arc::new((Mutex::new(ticket), Condvar::new()));

        let monitor_clone = monitor.clone();
        let (lock, cvar) = &*monitor;
        let (lock_clone, cvar_clone) = &*monitor_clone;

        let _ = coffemachine
            .notify_new_ticket(lock_clone, cvar_clone, new_ticket)
            .unwrap();

        if let Ok(guard) = lock.lock() {
            match cvar.wait_while(guard, |ticket: &mut Ticket| ticket.is_not_ready()) {
                Ok(result) => {
                    assert!(!result.is_not_ready());
                    assert_eq!(result.get_coffe_amount(), 10)
                }
                Err(_) => assert!(false),
            };
        } else {
            assert!(false)
        };
    }

    #[test]
    fn it_should_initilize_dispensers() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let machine_monitor = Arc::new((Mutex::new(false), Condvar::new()));
        let ticket_monitor = Arc::new((Mutex::new(Ticket::new(0)), Condvar::new()));

        let dispensers = coffemachine.init_dispensers(machine_monitor, ticket_monitor);
        assert_eq!(dispensers.len(), 1)
    }
}

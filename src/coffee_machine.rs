use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use std_semaphore::Semaphore;

use crate::{
    containers::{coffee_container::CoffeContainer, container::Container, resourse::Resourse},
    dispensers::dispenser::Dispenser,
    helpers::{
        ingredients::Ingredients,
        order_manager::{self, OrderManager},
        ticket::Ticket,
    },
};

const DISPENSERS: i32 = 2;
const INGREDIENTS: [Ingredients; 5] = [
    Ingredients::Coffee,
    Ingredients::Milk,
    Ingredients::Water,
    Ingredients::Foam,
    Ingredients::Cacao,
];

pub struct CoffeMachine {
    coffe_made: i32,
    req_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
    res_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
    bussy_sem: HashMap<Ingredients, Arc<Semaphore>>,
}

impl CoffeMachine {
    pub fn new() -> Self {
        let coffe_made = 0;
        let req_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>> = HashMap::new();
        let res_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>> = HashMap::new();
        let bussy_sem: HashMap<Ingredients, Arc<Semaphore>> = HashMap::new();

        Self {
            coffe_made,
            req_monitors,
            res_monitors,
            bussy_sem,
        }
    }

    fn init_containers(&mut self) -> Vec<JoinHandle<()>> {
        let mut containers = Vec::with_capacity(INGREDIENTS.len());

        for i in INGREDIENTS.iter().copied() {
            let req_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
            let res_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
            let sem = Arc::new(Semaphore::new(1));

            let request_monitor = req_monitor.clone();
            let response_monitor = res_monitor.clone();
            let sem_clone = sem.clone();

            self.req_monitors.insert(i, req_monitor);
            self.res_monitors.insert(i, res_monitor);
            self.bussy_sem.insert(i, sem);

            match i {
                Ingredients::Coffee => {
                    containers.push(thread::spawn(move || {
                        let mut coffe_container = CoffeContainer::new();
                        coffe_container.start(request_monitor, response_monitor, sem_clone);
                    }));
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
        order_lock: Arc<(Mutex<OrderManager>, Condvar)>,
    ) -> Vec<JoinHandle<()>> {
        let mut dispensers = Vec::with_capacity(DISPENSERS as usize);

        for i in 0..DISPENSERS {
            let req_monitors = self.req_monitors.clone();
            let res_monitors = self.res_monitors.clone();
            let order_monitor = order_lock.clone();
            let sems = self.bussy_sem.clone();

            dispensers.push(thread::spawn(move || {
                println!("creating dispenser {}", i);
                let dispenser = Dispenser::new(i);
                dispenser.start(order_monitor, &req_monitors, &res_monitors, &sems);
            }));
        }

        dispensers
    }

    fn notify_new_ticket(
        &self,
        lock: &Mutex<OrderManager>,
        cvar: &Condvar,
        mut new_ticket: Ticket,
    ) -> Result<(), String> {
        if let Ok(mut ticket_vec) = lock.lock() {
            new_ticket.ready_to_read();
            ticket_vec.add(new_ticket);
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
        Some(Ticket::new(i * 2))
    }

    fn kill_dispensers(&self, dispensers: Vec<JoinHandle<()>>) {
        for d in dispensers {
            if d.join().is_ok() {
                println!("[global]  - dispensers killed")
            };
        }
    }

    pub fn start(&mut self) {
        let order_manager = Arc::new((Mutex::new(OrderManager::new()), Condvar::new()));
        let _containers = self.init_containers();
        let dispensers = self.init_dispensers(order_manager.clone());
        let (order_lock, cvar) = &*order_manager;

        for i in 1..4 {
            match self.read_ticket(i) {
                Some(ticket) => match self.notify_new_ticket(order_lock, cvar, ticket) {
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
        helpers::{
            order_manager::OrderManager,
            ticket::{self, Ticket},
        },
    };

    #[test]
    fn it_should_initialize_with_0_coffe_made() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        assert_eq!(coffemachine.coffe_made, 0);
    }

    #[test]
    fn it_should_signal_coffe_dispenser() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let new_ticket = Ticket::new(10);
        let q = OrderManager::new();
        let monitor = Arc::new((Mutex::new(q), Condvar::new()));
        let (order_lock, cvar) = &*monitor;

        let _ = coffemachine
            .notify_new_ticket(order_lock, cvar, new_ticket)
            .unwrap();

        if let Ok(mut order_manager) =
            cvar.wait_while(order_lock.lock().unwrap(), |status| status.empty())
        {
            if let Some(order) = order_manager.extract() {
                assert!(!order.is_not_ready())
            }
        };
    }

    #[test]
    fn it_should_initilize_dispensers() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let q = OrderManager::new();
        let monitor = Arc::new((Mutex::new(q), Condvar::new()));

        let dispensers = coffemachine.init_dispensers(monitor);
        assert_eq!(dispensers.len(), DISPENSERS as usize)
    }
}

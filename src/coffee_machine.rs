use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use std_semaphore::Semaphore;

use crate::{
    containers::{
        cacao_container::CacaoContainer, coffee_container::CoffeContainer, container::Container,
        water_container::WaterContainer,
    },
    dispensers::dispenser::Dispenser,
    helpers::{
        ingredients::Ingredients,
        order::Order,
        order_manager::OrderManager,
        order_reader::{self, OrderReader},
        resourse::Resourse,
        stats_presenter::StatsPresenter,
    },
};

const DISPENSERS: i32 = 2;
const TIME: u64 = 5;
const INGREDIENTS: [Ingredients; 5] = [
    Ingredients::Coffee,
    Ingredients::Milk,
    Ingredients::Water,
    Ingredients::Foam,
    Ingredients::Cacao,
];
const PATH: &str = "res/orders.test1.json";
const END: i32 = -1;

pub struct CoffeMachine {
    req_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
    res_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
    bussy_sem: HashMap<Ingredients, Arc<Semaphore>>,
}

impl CoffeMachine {
    pub fn new() -> Self {
        let req_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>> = HashMap::new();
        let res_monitors: HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>> = HashMap::new();
        let bussy_sem: HashMap<Ingredients, Arc<Semaphore>> = HashMap::new();

        Self {
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
                Ingredients::Cacao => {
                    containers.push(thread::spawn(move || {
                        let mut water_container = CacaoContainer::new();
                        water_container.start(request_monitor, response_monitor, sem_clone);
                    }));
                }
                Ingredients::Water => {
                    containers.push(thread::spawn(move || {
                        let mut water_container = WaterContainer::new();
                        water_container.start(request_monitor, response_monitor, sem_clone);
                    }));
                }
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
        mut new_ticket: Order,
    ) -> Result<(), String> {
        if let Ok(mut ticket_vec) = lock.lock() {
            new_ticket.ready_to_read();
            ticket_vec.add(new_ticket);
            cvar.notify_all();
            println!("[coffee machine] - send new order");
            return Ok(());
        };
        Err("[error] - ticket monitor failed".to_string())
    }

    fn init_stat_presenter(
        &mut self,
        dispensers: &mut Vec<JoinHandle<()>>,
        order_lock: Arc<(Mutex<OrderManager>, Condvar)>,
    ) {
        dispensers.push(thread::spawn(move || {
            let dispenser = StatsPresenter::new(TIME);
            dispenser.start(order_lock);
        }));
    }

    fn read_ticket(&self, reader: &mut OrderReader) -> Option<Order> {
        match reader.get_order() {
            Some(o) => Some(o),
            None => None,
        }
    }

    fn kill_dispensers(&self, dispensers: Vec<JoinHandle<()>>) {
        for d in dispensers {
            if d.join().is_ok() {
                println!("[global]  - dispenser killed")
            };
        }
    }

    fn kill_containers(&self, containers: Vec<JoinHandle<()>>) {
        for i in INGREDIENTS.iter().copied() {
            if let Some(sem) = self.bussy_sem.get(&i) {
                sem.acquire();
                if let Some(monitor) = self.req_monitors.get(&i) {
                    let (lock_req, cvar) = monitor.as_ref();
                    if let Ok(mut old_resourse) = lock_req.lock() {
                        *old_resourse = Resourse::new(END);
                        old_resourse.ready_to_read();
                        cvar.notify_all();
                    };
                }
            }
        }

        for d in containers {
            if d.join().is_ok() {
                println!("[global]  - container killed")
            };
        }
    }

    pub fn start(&mut self) {
        let order_manager = Arc::new((Mutex::new(OrderManager::new()), Condvar::new()));
        let _containers = self.init_containers();
        let mut dispensers = self.init_dispensers(order_manager.clone());
        let mut order_reader = OrderReader::new(PATH.to_string());
        order_reader
            .read_json()
            .expect("[cofee machine] - Fail reading json file");
        self.init_stat_presenter(&mut dispensers, order_manager.clone());

        let (order_lock, cvar) = &*order_manager;
        loop {
            match self.read_ticket(&mut order_reader) {
                Some(ticket) => match self.notify_new_ticket(order_lock, cvar, ticket) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                },
                None => {
                    println!("[coffee machine] - no more orders.");
                    break;
                }
            }
        }
        println!("[coffe machine] - waiting for dispenseres be killed .");
        self.kill_dispensers(dispensers);
        self.kill_containers(_containers);
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

    use crate::{
        coffee_machine::{CoffeMachine, DISPENSERS},
        helpers::{order::Order, order_manager::OrderManager},
    };

    #[test]
    fn it_should_signal_coffe_dispenser() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let new_ticket = Order::new(10, 10, 10);
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

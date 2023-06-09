use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use std_semaphore::Semaphore;

use crate::{
    containers::{
        cacao_container::CacaoContainer, coffee_container::CoffeContainer,
        coffee_grain_container::CoffeeGrainContainer, container::Container,
        foam_container::FoamContainer, milk_container::MilkContainer,
        water_container::WaterContainer,
    },
    dispensers::dispenser::Dispenser,
    helpers::{
        container_message::{ContainerMessage, ContainerMessageType},
        ingredients::Ingredients,
        order::Order,
        order_manager::OrderManager,
        order_reader::OrderReader,
        stats_presenter::StatsPresenter,
    },
};

const TIME: u64 = 5;
const INGREDIENTS: [Ingredients; 6] = [
    Ingredients::CoffeGrain,
    Ingredients::Coffee,
    Ingredients::Milk,
    Ingredients::Water,
    Ingredients::Foam,
    Ingredients::Cacao,
];
const END: i32 = -1;

pub struct CoffeMachine {
    path: String,
    n_dispensers: i32,
    req_monitors: HashMap<Ingredients, Arc<(Mutex<ContainerMessage>, Condvar)>>,
    res_monitors: HashMap<Ingredients, Arc<(Mutex<ContainerMessage>, Condvar)>>,
    data_mutex: HashMap<Ingredients, i32>,
    bussy_sem: HashMap<Ingredients, Arc<Semaphore>>,
}

impl CoffeMachine {
    pub fn new(path: String, n_dispensers: i32) -> Self {
        let req_monitors: HashMap<Ingredients, Arc<(Mutex<ContainerMessage>, Condvar)>> =
            HashMap::new();
        let res_monitors: HashMap<Ingredients, Arc<(Mutex<ContainerMessage>, Condvar)>> =
            HashMap::new();
        let bussy_sem: HashMap<Ingredients, Arc<Semaphore>> = HashMap::new();
        let mut data_mutex: HashMap<Ingredients, i32> = HashMap::new();
        for i in INGREDIENTS.iter().copied() {
            data_mutex.insert(i, 0);
        }

        Self {
            path,
            n_dispensers,
            req_monitors,
            res_monitors,
            data_mutex,
            bussy_sem,
        }
    }

    fn init_containers(
        &mut self,
        mutex: Arc<Mutex<HashMap<Ingredients, i32>>>,
    ) -> Vec<JoinHandle<()>> {
        let mut containers = Vec::with_capacity(INGREDIENTS.len());

        for i in INGREDIENTS.iter().copied() {
            let req_monitor = Arc::new((
                Mutex::new(ContainerMessage::new(
                    0,
                    ContainerMessageType::ResourseRequest,
                )),
                Condvar::new(),
            ));
            let res_monitor = Arc::new((
                Mutex::new(ContainerMessage::new(
                    0,
                    ContainerMessageType::ResourseRequest,
                )),
                Condvar::new(),
            ));
            let sem = Arc::new(Semaphore::new(1));

            let request_monitor = req_monitor.clone();
            let response_monitor = res_monitor.clone();
            let sem_clone = sem.clone();

            self.req_monitors.insert(i, req_monitor);
            self.res_monitors.insert(i, res_monitor);
            self.bussy_sem.insert(i, sem);
            let d_mutex = mutex.clone();
            match i {
                Ingredients::Coffee => {
                    let req = self
                        .req_monitors
                        .get(&Ingredients::CoffeGrain)
                        .expect("COFFE GRAIN REQ MONITOR NOT FOUND")
                        .to_owned();
                    let res = self
                        .res_monitors
                        .get(&Ingredients::CoffeGrain)
                        .expect("COFFE GRAIN RES MONITOR NOT FOUND")
                        .to_owned();
                    let s = self
                        .bussy_sem
                        .get(&Ingredients::CoffeGrain)
                        .expect("COFFE GRAIN SEMAPHORE NOT FOUND")
                        .to_owned();

                    containers.push(thread::spawn(move || {
                        let mut coffe_container =
                            CoffeContainer::new(req.clone(), res.clone(), s.clone());
                        coffe_container.start(
                            request_monitor,
                            response_monitor,
                            sem_clone,
                            d_mutex,
                        );
                    }));
                }
                Ingredients::CoffeGrain => {
                    containers.push(thread::spawn(move || {
                        let mut coffe_container: CoffeeGrainContainer = CoffeeGrainContainer::new();
                        coffe_container.start(
                            request_monitor,
                            response_monitor,
                            sem_clone,
                            d_mutex,
                        );
                    }));
                }
                Ingredients::Milk => {
                    containers.push(thread::spawn(move || {
                        let mut water_container: MilkContainer = MilkContainer::new();
                        water_container.start(
                            request_monitor,
                            response_monitor,
                            sem_clone,
                            d_mutex,
                        );
                    }));
                }
                Ingredients::Cacao => {
                    containers.push(thread::spawn(move || {
                        let mut water_container = CacaoContainer::new();
                        water_container.start(
                            request_monitor,
                            response_monitor,
                            sem_clone,
                            d_mutex,
                        );
                    }));
                }
                Ingredients::Water => {
                    containers.push(thread::spawn(move || {
                        let mut water_container = WaterContainer::new();
                        water_container.start(
                            request_monitor,
                            response_monitor,
                            sem_clone,
                            d_mutex,
                        );
                    }));
                }
                Ingredients::Foam => {
                    let req = self
                        .req_monitors
                        .get(&Ingredients::Milk)
                        .expect("MILK REQ MONITOR NOT FOUND")
                        .to_owned();
                    let res = self
                        .res_monitors
                        .get(&Ingredients::Milk)
                        .expect("MILK RES MONITOR NOT FOUND")
                        .to_owned();
                    let s = self
                        .bussy_sem
                        .get(&Ingredients::Milk)
                        .expect("MILK SEMAPHORE NOT FOUND")
                        .to_owned();
                    containers.push(thread::spawn(move || {
                        let mut foam_container =
                            FoamContainer::new(req.clone(), res.clone(), s.clone());
                        foam_container.start(request_monitor, response_monitor, sem_clone, d_mutex);
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
        let mut dispensers = Vec::with_capacity(self.n_dispensers as usize);

        for i in 0..self.n_dispensers {
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
            println!("[coffee machine] - notify new order");
            return Ok(());
        };
        Err("[error] - ticket monitor failed".to_string())
    }

    fn init_stat_presenter(
        &mut self,
        dispensers: &mut Vec<JoinHandle<()>>,
        order_lock: Arc<(Mutex<OrderManager>, Condvar)>,
        d_mutex: Arc<Mutex<HashMap<Ingredients, i32>>>,
    ) {
        dispensers.push(thread::spawn(move || {
            let dispenser = StatsPresenter::new(TIME);
            dispenser.start(order_lock, d_mutex);
        }));
    }

    fn read_ticket(&self, reader: &mut OrderReader) -> Option<Order> {
        reader.get_order()
    }

    fn kill_dispensers(&self, dispensers: Vec<JoinHandle<()>>) {
        for d in dispensers {
            if d.join().is_ok() {
                println!("[global]  - dispenser killed")
            };
        }
    }

    fn kill_containers(&self, containers: Vec<JoinHandle<()>>) {
        println!("[global]  - notifing containers to stop");
        for i in INGREDIENTS.iter() {
            if let Some(sem) = self.bussy_sem.get(i) {
                sem.acquire();
                if let Some(monitor) = self.req_monitors.get(i) {
                    let (lock_req, cvar) = monitor.as_ref();
                    if let Ok(mut old_resourse) = lock_req.lock() {
                        *old_resourse =
                            ContainerMessage::new(END, ContainerMessageType::KillRequest);
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
        let d_mutex = Arc::new(Mutex::new(self.data_mutex.clone()));
        let containers = self.init_containers(d_mutex.clone());
        let mut dispensers = self.init_dispensers(order_manager.clone());
        let mut order_reader = OrderReader::new(self.path.clone());
        order_reader
            .read_json()
            .expect("[cofee machine] - Failed reading json file");
        self.init_stat_presenter(&mut dispensers, order_manager.clone(), d_mutex);

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
                    println!("[coffee machine] - no more orders to process.");
                    break;
                }
            }
        }
        self.kill_dispensers(dispensers);
        self.kill_containers(containers);
    }
}

#[cfg(test)]
mod coffemachine_test {
    use std::sync::{Arc, Condvar, Mutex};

    use crate::{
        coffee_machine::CoffeMachine,
        helpers::{order::Order, order_manager::OrderManager},
    };

    #[test]
    fn it_should_signal_coffe_dispenser() {
        let coffemachine: CoffeMachine = CoffeMachine::new("text".to_string(), 2);
        let new_ticket = Order::new(10, 10, 10, 10, 0);
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
        let coffemachine: CoffeMachine = CoffeMachine::new("text".to_string(), 2);
        let q = OrderManager::new();
        let monitor = Arc::new((Mutex::new(q), Condvar::new()));

        let dispensers = coffemachine.init_dispensers(monitor);
        assert_eq!(dispensers.len(), 2)
    }
}

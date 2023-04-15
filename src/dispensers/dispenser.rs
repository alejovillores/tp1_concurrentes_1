use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Condvar, Mutex},
    thread::{self},
    time::Duration,
};

use std_semaphore::Semaphore;

use crate::{
    containers::resourse::Resourse,
    helpers::{ingredients::Ingredients, ticket::Ticket},
};

const FINISH_FLAG: i32 = -1;
const INGREDIENTS: [Ingredients; 5] = [
    Ingredients::Coffee,
    Ingredients::Milk,
    Ingredients::Water,
    Ingredients::Foam,
    Ingredients::Cacao,
];

#[derive(Debug)]
pub struct Dispenser {
    id: i32,
}

impl Dispenser {
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    fn process_order(
        &self,
        req_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
        res_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
        order: Ticket,
    ) -> i32 {
        let mut status = 0;
        for ingredient in INGREDIENTS.iter().copied() {
            match ingredient {
                Ingredients::Coffee => {
                    let resourse = Resourse::new(order.get_coffe_amount(), self.id);

                    if let Some(monitor) = req_monitors.get(&ingredient) {
                        let (lock_req, cvar_req) = monitor.as_ref();
                        println!(
                            "[dispenser {}] - send amount of {} coffee units to coffee container",
                            self.id,
                            resourse.get_amount()
                        );
                        if self.notify_container(lock_req, cvar_req, resourse).is_err() {
                            println!("[dispenser {}] fail requesting resourse", self.id)
                        }
                    }

                    if let Some(monitor) = res_monitors.get(&ingredient) {
                        let (res_lock, res_cvar) = monitor.as_ref();
                        if let Ok(coffee_delivered) = self.wait_coffe_container(res_lock, res_cvar)
                        {
                            if coffee_delivered == FINISH_FLAG {
                                status = FINISH_FLAG;
                            } else if self.dispense(coffee_delivered).is_err() {
                                println!("[dispenser {}] fail dispensign coffee", self.id)
                            }
                        }
                    }
                }
                Ingredients::CoffeGrain => todo!(),
                Ingredients::Milk => println!("[dispenser {}] no milk", self.id),
                Ingredients::Foam => println!("[dispenser {}] no foam", self.id),
                Ingredients::Cacao => println!("[dispenser {}] no cacao", self.id),
                Ingredients::Water => println!("[dispenser {}] no hot water", self.id),
            }
        }
        status
    }

    // Simulate dispense time
    fn dispense(&self, amount: i32) -> Result<(), std::fmt::Error> {
        println!("[dispenser {}] - dispensing {} units", self.id, amount);

        thread::sleep(Duration::from_secs(amount as u64));

        println!("[dispenser {}] - finished dispensing", self.id);
        Ok(())
    }

    // Waits form a new ticket from coffee machine
    fn wait_new_ticket(&self, lock: &Mutex<VecDeque<Ticket>>, sem: &Semaphore) -> Option<Ticket> {
        sem.acquire();
        if let Ok(mut ticket_vec) = lock.lock() {
            if let Some(mut order) = ticket_vec.pop_back() {
                order.read();
                println!("[dispenser {}] - new order  ", self.id);
                return Some(order);
            }
        };
        None
    }

    // Signal a container that amount of ingredient needed
    fn notify_container(
        &self,
        lock: &Mutex<Resourse>,
        cvar: &Condvar,
        resourse: Resourse,
    ) -> Result<(), String> {
        if let Ok(mut old_resourse) = lock.lock() {
            *old_resourse = resourse;
            old_resourse.ready_to_read();
            cvar.notify_all();
            return Ok(());
        };
        Err("[error] - coffee amount monitor failed in coffee dispenser".to_string())
    }

    // waits for coffee container to respond
    fn wait_coffe_container(&self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready(self.id))
            {
                let resourse_amount = resourse.get_amount();
                resourse.read();
                println!(
                    "[coffee dispenser] - response with {} units from container",
                    resourse.get_amount()
                );
                return Ok(resourse_amount);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    pub fn start(
        &self,
        machine_sem: Arc<Semaphore>,
        order_lock: Arc<Mutex<VecDeque<Ticket>>>,
        containers_req_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
        containers_res_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
    ) {
        loop {
            if let Some(order) = self.wait_new_ticket(&order_lock, &machine_sem) {
                let order_status =
                    self.process_order(containers_req_monitors, containers_res_monitors, order);

                if order_status == FINISH_FLAG {
                    println!("[dispenser {} ] - killing dispenser ", self.id);
                    break;
                }
            } else {
                println!("[dispenser {} ] - killing dispenser ", self.id);
                break;
            }
        }
    }
}

#[cfg(test)]
mod dispenser_test {
    use std::{
        collections::VecDeque,
        sync::{Arc, Condvar, Mutex},
    };

    use std_semaphore::Semaphore;

    use crate::{
        containers::resourse::Resourse, dispensers::dispenser::Dispenser, helpers::ticket::Ticket,
    };

    #[test]
    fn it_should_dispense_2_sec_of_coffe() {
        let dispenser = Dispenser::new(0);
        match dispenser.dispense(2) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_return_10_when_wait_new_ticket_is_ready() {
        let dispenser = Dispenser::new(0);
        let mut q = VecDeque::new();
        q.push_front(Ticket::new(10));
        let ticket = Arc::new(Mutex::new(q));

        let sem = Semaphore::new(1);

        match dispenser.wait_new_ticket(&ticket, &sem) {
            Some(new_ticket) => assert_eq!(new_ticket.get_coffe_amount(), 10),
            None => assert!(false),
        }
    }

    #[test]
    fn it_should_signal_when_new_coffe_amount_is_ready() {
        let dispenser = Dispenser::new(0);
        let resourse: Resourse = Resourse::new(0, dispenser.id);

        let monitor = Arc::new((Mutex::new(resourse), Condvar::new()));
        let monitor_clone = monitor.clone();
        let (lock, cvar) = &*monitor;
        let (lock_clone, cvar_clone) = &*monitor_clone;

        let mut new_resourse: Resourse = Resourse::new(20, dispenser.id);
        new_resourse.ready_to_read();

        dispenser
            .notify_container(lock, cvar, new_resourse)
            .unwrap();

        let resourse = cvar_clone
            .wait_while(lock_clone.lock().unwrap(), |status| {
                status.is_not_ready(dispenser.id)
            })
            .unwrap();
        assert_eq!(resourse.get_amount(), 20);
    }
}

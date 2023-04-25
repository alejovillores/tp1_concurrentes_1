use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread::{self},
    time::Duration,
};

use std_semaphore::Semaphore;

use crate::{
    helpers::resourse::Resourse,
    helpers::{ingredients::Ingredients, order::Order, order_manager::OrderManager, container_message::ContainerMessage},
};

const FINISH_FLAG: i32 = -1;
const NO_MORE: i32 = 0;

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
        order: Order,
        containers_sem: &HashMap<Ingredients, Arc<Semaphore>>,
    ) -> i32 {
        let mut status = FINISH_FLAG;
        for ingredient in INGREDIENTS.iter().copied() {
            match ingredient {
                Ingredients::Milk => println!("[dispenser {}] no milk", self.id),
                Ingredients::Foam => println!("[dispenser {}] no foam", self.id),
                Ingredients::CoffeGrain => {}
                _ => {
                    let amount = order.get_ingredient_amount(ingredient);
                    if amount > 0 {
                        let resourse = Resourse::new(amount);

                        if let Some(sem) = containers_sem.get(&ingredient) {
                            sem.acquire();
                            println!(
                                "[dispenser {}] has access to {:?} container",
                                self.id, ingredient
                            );
                            if let Ok(res) = self.process_ingredient(
                                req_monitors,
                                res_monitors,
                                resourse,
                                ingredient,
                            ) {
                                if res == NO_MORE {
                                    println!(
                                        "[dispenser {}] could not take amount needed from {:?} container",
                                        self.id, ingredient
                                    );
                                }
                                status = res;
                            }
                        }
                    }
                }
            }
        }
        status
    }

    // try to ask for an amount of ingredient and wait for response.
    fn process_ingredient(
        &self,
        req_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
        res_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
        resourse: Resourse,
        ingredient: Ingredients,
    ) -> Result<i32, String> {
        if let Some(monitor) = req_monitors.get(&ingredient) {
            let (lock_req, cvar_req) = monitor.as_ref();
            println!(
                "[dispenser {}] - send amount of {} , {:?} units to container",
                self.id,
                resourse.get_amount(),
                ingredient
            );
            if self.notify_container(lock_req, cvar_req, resourse).is_err() {
                println!("[dispenser {}] fail requesting resourse", self.id)
            }
        }

        if let Some(monitor) = res_monitors.get(&ingredient) {
            let (res_lock, res_cvar) = monitor.as_ref();
            if let Ok(res_delivered) = self.wait_container(res_lock, res_cvar) {
                if res_delivered == FINISH_FLAG {
                    return Ok(FINISH_FLAG);
                } else if self.dispense(res_delivered).is_err() {
                    println!("[dispenser {}] fail dispensign {:?}", self.id, ingredient);
                    return Err("[error] - dispenser resourse monitor failed".to_string());
                } else {
                    return Ok(res_delivered);
                }
            }
            return Err("[error] - dispenser resourse monitor failed".to_string());
        }

        Err("[error] - dispenser resourse no monitor found".to_string())
    }

    // Simulate dispense time
    fn dispense(&self, amount: i32) -> Result<(), std::fmt::Error> {
        println!("[dispenser {}] - dispensing {} units", self.id, amount);

        thread::sleep(Duration::from_secs(amount as u64));

        println!("[dispenser {}] - finished dispensing", self.id);
        Ok(())
    }

    // Waits form a new ticket from coffee machine
    fn wait_new_ticket(&self, lock: &Mutex<OrderManager>, cvar: &Condvar) -> Option<Order> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut order_manager) = cvar.wait_while(guard, |status| status.empty()) {
                if let Some(mut order) = order_manager.extract() {
                    order.read();
                    println!("[dispenser {}] - new order  ", self.id);
                    return Some(order);
                } else {
                    println!("[dispenser {} ] - no more orders", self.id);
                    return None;
                }
            }
        }
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
    fn wait_container(&self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let resourse_amount = resourse.get_amount();
                resourse.read();
                println!(
                    "[dispenser {} ] - response with {} units from container",
                    self.id,
                    resourse.get_amount()
                );
                return Ok(resourse_amount);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    pub fn start(
        &self,
        order_monitor: Arc<(Mutex<OrderManager>, Condvar)>,
        containers_req_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
        containers_res_monitors: &HashMap<Ingredients, Arc<(Mutex<Resourse>, Condvar)>>,
        containers_sem: &HashMap<Ingredients, Arc<Semaphore>>,
    ) {
        loop {
            let (order_lock, cvar) = &*order_monitor;
            if let Some(order) = self.wait_new_ticket(order_lock, cvar) {
                let _ = self.process_order(
                    containers_req_monitors,
                    containers_res_monitors,
                    order,
                    containers_sem,
                );

                println!("[dispenser {} ] - dispenser finished order ", self.id);
            } else {
                println!("[dispenser {} ] - killing dispenser ", self.id);
                break;
            }
        }
    }
}

#[cfg(test)]
mod dispenser_test {
    use std::sync::{Arc, Condvar, Mutex};

    use crate::{
        dispensers::dispenser::Dispenser,
        helpers::{ingredients::Ingredients, resourse::Resourse, container_message::ContainerMessage},
        helpers::{order::Order, order_manager::OrderManager},
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
        let mut q = OrderManager::new();
        q.add(Order::new(10, 10, 10, 10));

        let ticket = Arc::new((Mutex::new(q), Condvar::new()));
        let (order_lock, cvar) = &*ticket;

        match dispenser.wait_new_ticket(order_lock, cvar) {
            Some(new_ticket) => {
                assert_eq!(new_ticket.get_ingredient_amount(Ingredients::Coffee), 10)
            }
            None => assert!(false),
        }
    }

    #[test]
    fn it_should_signal_when_new_coffe_amount_is_ready() {
        let dispenser = Dispenser::new(0);
        let resourse: Resourse = Resourse::new(0);

        let monitor = Arc::new((Mutex::new(resourse), Condvar::new()));
        let monitor_clone = monitor.clone();
        let (lock, cvar) = &*monitor;
        let (lock_clone, cvar_clone) = &*monitor_clone;

        let mut new_resourse: Resourse = Resourse::new(20);
        new_resourse.ready_to_read();

        dispenser
            .notify_container(lock, cvar, new_resourse)
            .unwrap();

        let resourse = cvar_clone
            .wait_while(lock_clone.lock().unwrap(), |status| status.is_not_ready())
            .unwrap();
        assert_eq!(resourse.get_amount(), 20);
    }
}

use super::{
    coffee_grain_container::CoffeeGrainContainer, container::Container, resourse::Resourse,
};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

const CAPACITY: i32 = 100;
const FINISH_FLAG: i32 = -1;

pub struct CoffeContainer {
    capacity: i32,
    last_dispenser_id: i32,
}

impl CoffeContainer {
    pub fn new() -> Self {
        let capacity = 0;
        let last_dispenser_id = 0;
        Self {
            capacity,
            last_dispenser_id,
        }
    }

    // Attempst to refill container
    fn refill(
        &mut self,
        refill_req_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        refill_amount: i32,
    ) {
        // ask
        let req_resourse = Resourse::new(refill_amount, self.last_dispenser_id);
        let (req_lock, req_cvar) = &*refill_req_monitor;
        self.notify_container(req_lock, req_cvar, req_resourse);

        // wait
        let (res_lock, res_cvar) = &*refill_res_monitor;
        if let Ok(amount) = self.wait_container_coffee(res_lock, res_cvar) {
            println!("[coffee container] - starting refill");
            if amount == FINISH_FLAG {
                println!("[coffe container] - out of coffe");
                self.capacity = FINISH_FLAG
            } else {
                self.capacity += amount;
                println!("[coffee container] - refill complete");
            }
        }
    }

    // Consume capacity or ask for refill
    fn consume(
        &mut self,
        refill_req_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        amount: i32,
    ) -> Result<i32, String> {
        if self.capacity >= amount && amount.is_positive() {
            self.capacity -= amount;
            return Ok(amount);
        }
        if self.capacity == FINISH_FLAG {
            return Ok(FINISH_FLAG);
        }

        if amount.is_negative() {
            self.notify_end_message(refill_req_monitor);
            return Ok(FINISH_FLAG);
        } else {
            let refill_amount = CAPACITY - self.capacity;
            self.refill(
                refill_req_monitor.clone(),
                refill_res_monitor.clone(),
                refill_amount,
            );
            self.consume(refill_req_monitor, refill_res_monitor, amount)
        }
    }

    // Waits for dispenser to send new coffee request
    fn wait_dispenser_coffee(
        &mut self,
        lock: &Mutex<Resourse>,
        cvar: &Condvar,
    ) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            let id = self.last_dispenser_id;
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready(id)) {
                let coffe_amount = resourse.get_amount();
                self.last_dispenser_id = resourse.get_dispenser_id();

                if coffe_amount == FINISH_FLAG {
                    println!(
                        "[coffee container] - dispenser {} asking sending FINISHING FLAG",
                        self.last_dispenser_id
                    );
                } else {
                    println!(
                        "[coffee container] - dispenser {} asking for {} units of coffee",
                        self.last_dispenser_id, coffe_amount
                    );
                }
                resourse.read();
                return Ok(coffe_amount);
            }
        };
        Err("[error] - coffee container  monitor failed".to_string())
    }

    // Notify dispenser about new resourse avaliable
    fn notify_dispenser(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar, res: Resourse) {
        if let Ok(mut resourse) = lock.lock() {
            *resourse = res;
            resourse.ready_to_read();
            if resourse.get_amount() == FINISH_FLAG {
                println!(
                    "[coffee container] - notifying dispenser {}  FINISHING FLAG ",
                    self.last_dispenser_id
                );
            } else {
                println!(
                    "[coffee container] - sending {} units to dispenser with id: {}",
                    resourse.get_amount(),
                    self.last_dispenser_id
                );
            }
            cvar.notify_all();
        }
    }

    // Waits for grain contaniner to send new coffee response
    fn wait_container_coffee(
        &mut self,
        lock: &Mutex<Resourse>,
        cvar: &Condvar,
    ) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            let id = guard.get_dispenser_id();
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready(id)) {
                let coffe_amount = resourse.get_amount();

                if coffe_amount == FINISH_FLAG {
                    println!("[coffee container] - received FINISH FLAG units from coffee grain container");
                } else {
                    println!(
                        "[coffee container] - received {} units from coffee grain container",
                        coffe_amount
                    );
                }

                resourse.read();
                return Ok(coffe_amount);
            }
        };
        Err(
            "[error] - coffee container  monitor failed waiting for grain container response"
                .to_string(),
        )
    }

    // Notify container about new resourse request
    fn notify_container(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar, res: Resourse) {
        if let Ok(mut resourse) = lock.lock() {
            *resourse = res;
            println!(
                "[coffee container] - sending request of {} units to coffee grain container",
                resourse.get_amount()
            );
            resourse.ready_to_read();
            cvar.notify_all();
        }
    }

    // Create coffee grain container
    fn init_container(
        &self,
        refill_req_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<Resourse>, Condvar)>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut coffee_grain_container = CoffeeGrainContainer::new();
            coffee_grain_container.start(refill_req_monitor, refill_res_monitor);
        })
    }

    fn notify_end_message(&mut self, refill_req_monitor: Arc<(Mutex<Resourse>, Condvar)>) {
        // send
        let req_resourse = Resourse::new(FINISH_FLAG, self.last_dispenser_id);
        let (req_lock, req_cvar) = &*refill_req_monitor;
        self.notify_container(req_lock, req_cvar, req_resourse);
    }

    // Kill child thread container
    fn kill_container(&self, coffee_grain_handler: JoinHandle<()>) {
        if coffee_grain_handler.join().is_ok() {
            println!("[global]  - coffee container grained killed")
        };
    }
}

impl Container for CoffeContainer {
    fn start(
        &mut self,
        dispenser_req_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        dispenser_res_monitor: Arc<(Mutex<Resourse>, Condvar)>,
    ) {
        let refill_req_monitor = Arc::new((Mutex::new(Resourse::new(0, 0)), Condvar::new()));
        let refill_res_monitor = Arc::new((Mutex::new(Resourse::new(0, 0)), Condvar::new()));
        let grain_container =
            self.init_container(refill_req_monitor.clone(), refill_res_monitor.clone());

        loop {
            let (lock, cvar) = &*dispenser_req_monitor;
            if let Ok(res) = self.wait_dispenser_coffee(lock, cvar) {
                println!("[coffee container] - attempting to consume amount {}", res);

                if let Ok(amounte_consumed) =
                    self.consume(refill_req_monitor.clone(), refill_res_monitor.clone(), res)
                {
                    let (res_lock, res_cvar) = &*dispenser_res_monitor;
                    self.notify_dispenser(
                        res_lock,
                        res_cvar,
                        Resourse::new(amounte_consumed, self.last_dispenser_id),
                    );
                    if res == FINISH_FLAG {
                        self.kill_container(grain_container);
                        println!("[coffee container] - finishing ");
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod coffecontainer_test {
    use std::sync::{Arc, Condvar, Mutex};

    use crate::containers::{
        coffee_container::{CoffeContainer, CAPACITY},
        resourse::Resourse,
    };

    #[test]
    fn it_should_init_with_0() {
        let coffee_container = CoffeContainer::new();
        assert_eq!(coffee_container.capacity, 0)
    }

    #[test]
    fn it_should_refill_with_0_capacity() {
        let refill_req_monitor = Arc::new((Mutex::new(Resourse::new(100, 1)), Condvar::new()));
        let refill_res_monitor = Arc::new((Mutex::new(Resourse::new(100, 1)), Condvar::new()));
        let mut coffee_container = CoffeContainer::new();
        coffee_container.capacity = 0;

        let (res_lock, res_cvar) = &*refill_res_monitor;
        if let Ok(mut resourse) = res_lock.lock() {
            resourse.ready_to_read();
            res_cvar.notify_all();
        }
        coffee_container.refill(refill_req_monitor, refill_res_monitor.clone(), 100);

        assert_eq!(coffee_container.capacity, CAPACITY)
    }

    #[test]
    fn it_should_has_value_with_valid_amount() {
        let mut coffee_container = CoffeContainer::new();
        let mut res = Resourse::new(10, 0);
        res.ready_to_read();
        let monitor = Arc::new((Mutex::new(res), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffee_container.wait_dispenser_coffee(lock, cvar) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }
}

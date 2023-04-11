use super::{
    coffee_grain_container::CoffeeGrainContainer, container::Container, resourse::Resourse,
};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

const CAPACITY: i32 = 100;
const FINISH_FLAG: i32 = -1;

pub struct CoffeContainer {
    capacity: i32,
}

impl CoffeContainer {
    pub fn new() -> Self {
        let capacity = CAPACITY;
        Self { capacity }
    }

    // Attempst to refill container
    fn refill(
        &mut self,
        refill_req_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        refill_amount: i32,
    ) {
        // ask
        let req_resourse = Resourse::new(refill_amount);
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
            }
        }
    }

    // Consume capacity or ask for refill
    fn consume(
        &mut self,
        refill_req_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        mut amount: i32,
    ) -> Result<i32, String> {
        
        if self.capacity >= amount && amount.is_positive(){
            self.capacity -= amount;
            return Ok(amount);
        }
        if self.capacity == FINISH_FLAG {
            return Ok(FINISH_FLAG);
        } 
        
        if amount.is_negative() {
            self.refill(
                refill_req_monitor.clone(),
                refill_res_monitor.clone(),
                FINISH_FLAG,
            );
            return Ok(FINISH_FLAG)            
        }else {
            let refill_amount = CAPACITY - self.capacity;
            self.refill(
                refill_req_monitor.clone(),
                refill_res_monitor.clone(),
                refill_amount,
            );
            amount -= self.capacity;
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
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let coffe_amount = resourse.get_amount();
                println!("[coffee container] - dispenser asking for {} units of coffe", coffe_amount);
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
            println!("[coffee container] - notifying dispenser");
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
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let coffe_amount = resourse.get_amount();
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
        println!("[coffee container] - sending request to container");
        if let Ok(mut resourse) = lock.lock() {
            *resourse = res;
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
        let coffee_grain_handler = thread::spawn(move || {
            let mut coffee_grain_container = CoffeeGrainContainer::new();
            coffee_grain_container.start(refill_req_monitor, refill_res_monitor);
        });

        coffee_grain_handler
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
        let refill_req_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
        let refill_res_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
        let grain_container =
            self.init_container(refill_req_monitor.clone(), refill_res_monitor.clone());

        loop {
            let (lock, cvar) = &*dispenser_req_monitor;
            if let Ok(amount) = self.wait_dispenser_coffee(lock, cvar) {
                println!("[coffee container] - attempting to consume amount {}", amount);
                if let Ok(amounte_consumed) = self.consume(
                    refill_req_monitor.clone(),
                    refill_res_monitor.clone(),
                    amount,
                ) {
                    let (res_lock, res_cvar) = &*dispenser_res_monitor;
                    self.notify_dispenser(res_lock, res_cvar, Resourse::new(amounte_consumed));
                    if amount == FINISH_FLAG {
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
        resourse::{Resourse},
    };

    #[test]
    fn it_should_init_with_30_capacity() {
        let coffee_container = CoffeContainer::new();
        assert_eq!(coffee_container.capacity, CAPACITY)
    }

    #[test]
    fn it_should_refill_with_0_capacity() {
        let refill_req_monitor = Arc::new((Mutex::new(Resourse::new(100)), Condvar::new()));
        let refill_res_monitor = Arc::new((Mutex::new(Resourse::new(100)), Condvar::new()));
        let mut coffee_container = CoffeContainer::new();
        coffee_container.capacity = 0;

        
        let (res_lock, res_cvar) = &*refill_res_monitor;
        if let Ok( mut resourse) = res_lock.lock() {
            resourse.ready_to_read();
            res_cvar.notify_all();
        }
        coffee_container.refill(refill_req_monitor, refill_res_monitor.clone(), 100);

        assert_eq!(coffee_container.capacity, CAPACITY)
    }

    #[test]
    fn it_should_has_value_with_valid_amount() {
        let mut coffee_container = CoffeContainer::new();
        let mut res = Resourse::new(10);
        res.ready_to_read();
        let monitor = Arc::new((Mutex::new(res), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffee_container.wait_dispenser_coffee(lock, cvar) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }
}

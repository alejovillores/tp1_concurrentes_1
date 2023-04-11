use super::{
    coffee_grain_container::CoffeeGrainContainer, container::Container, resourse::Resourse,
};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};
use std_semaphore::Semaphore;

const CAPACITY: i32 = 100;

pub struct CoffeContainer {
    capacity: i32,
    dispenser_semaphore: Arc<Semaphore>,
}

impl CoffeContainer {
    pub fn new(dispenser_semaphore: Arc<Semaphore>) -> Self {
        let capacity = CAPACITY;
        Self {
            capacity,
            dispenser_semaphore,
        }
    }

    fn refill(&mut self) {
        println!("[coffee container] - starting refill");
        thread::sleep(Duration::from_secs(2));
        self.capacity = CAPACITY;
    }

    fn consume(&mut self, mut amount: i32) {
        if self.capacity >= amount {
            self.capacity -= amount;
        } else {
            self.refill();
            amount -= CAPACITY;
            self.consume(amount)
        }
    }

    fn get_coffe(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_ready()) {
                let coffe_amount = resourse.get_amount();
                resourse.ready();
                return Ok(coffe_amount);
            }
        };
        Err("[error] - coffee container  monitor failed".to_string())
    }

    fn signal_dispenser(&self) {
        println!("[cofee container] - releasing semaphore");
        self.dispenser_semaphore.release();
    }

    fn init_container(&self, _refill_monitor: Arc<(Mutex<Resourse>, Condvar)>) -> JoinHandle<()> {
        let container = thread::spawn(move || {
            let _coffee_grain_container = CoffeeGrainContainer::new();
            //let coffee_grain_container.start(refill_monitor);
        });

        container
    }

    fn _kill_container(&self, container: JoinHandle<()>) {
        if container.join().is_ok() {
            println!("[global]  - container killed")
        };
    }
}

impl Container for CoffeContainer {
    fn start(&mut self, monitor: Arc<(Mutex<Resourse>, Condvar)>) {
        let refill_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
        let _grain_container = self.init_container(refill_monitor);

        loop {
            let (lock, cvar) = &*monitor;
            if let Ok(amount) = self.get_coffe(lock, cvar) {
                if amount < 0 {
                    println!("[coffee container] - Finishing");
                    break;
                } else {
                    println!("[coffee container] - consuming amount");
                    self.consume(amount);
                    self.signal_dispenser();
                }
            }
        }
    }
}

#[cfg(test)]
mod coffecontainer_test {
    use std::sync::{Arc, Condvar, Mutex};

    use std_semaphore::Semaphore;

    use crate::containers::{
        coffe_container::{CoffeContainer, CAPACITY},
        resourse::Resourse,
    };

    #[test]
    fn it_should_init_with_30_capacity() {
        let sem = Arc::new(Semaphore::new(0));
        let coffee_container = CoffeContainer::new(sem);
        assert_eq!(coffee_container.capacity, CAPACITY)
    }

    #[test]
    fn it_should_refill_with_0_capacity() {
        let sem = Arc::new(Semaphore::new(0));
        let mut coffee_container = CoffeContainer::new(sem);
        coffee_container.capacity = 0;
        coffee_container.refill();
        assert_eq!(coffee_container.capacity, CAPACITY)
    }

    #[test]
    fn it_should_has_value_with_valid_amount() {
        let sem = Arc::new(Semaphore::new(0));
        let mut coffee_container = CoffeContainer::new(sem);
        let monitor = Arc::new((Mutex::new(Resourse::new(10)), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffee_container.get_coffe(lock, cvar) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_signal_semaphore() {
        let sem = Arc::new(Semaphore::new(0));
        let coffee_container = CoffeContainer::new(sem.clone());
        coffee_container.signal_dispenser();

        sem.acquire();
        assert!(true)
    }

    #[test]
    fn it_should_refill_with_bigger_amount() {
        let sem = Arc::new(Semaphore::new(0));
        let mut coffee_container = CoffeContainer::new(sem);
        let monitor = Arc::new((Mutex::new(Resourse::new(110)), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffee_container.get_coffe(lock, cvar) {
            Ok(c) => {
                coffee_container.consume(c);
                assert_eq!(coffee_container.capacity, 90)
            }
            Err(_) => assert!(false),
        }
    }
}

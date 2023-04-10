use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use std_semaphore::Semaphore;

use crate::{ticket::Ticket, containers::{resourse::Resourse, coffe_container::CoffeContainer}};

const END: i32 = 0;
const CONTAINERS: usize = 1;

#[derive(Debug)]
pub struct CoffeDispenser {}

impl CoffeDispenser {
    pub fn new() -> Self {
        Self {}
    }

 
    fn dispense(&self, amount: i32) -> Result<(), std::fmt::Error> {
        // TODO: should signal coffe container that amount of coffe is needed
        // TODO: should wait for coffe container to allow me.

        // imitate dispense
        println!("[coffee dispenser] - dispensing {} units of coffee", amount);
        thread::sleep(Duration::from_secs(amount as u64));
        println!("[coffee dispenser] - coffee amount dispensed");
        Ok(())
    }

    
    fn signal_finish(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<(), String> {
        if let Ok(mut dispenser) = lock.lock() {
            *dispenser = false;
            cvar.notify_all();
            println!("[coffee dispenser] - send signal_finish");
            return Ok(());
        };
        Err("[error] - ticket monitor failed in coffee dispenser".to_string())
    }


    fn new_ticket(&self, lock: &Mutex<Ticket>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            // As long as the value inside the `Mutex<bool>` is `true`, we wait
            if let Ok(ticket) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let coffe_amount = ticket.get_coffe_amount();
                if coffe_amount >= END {
                    println!("[coffee dispenser] - NEW TICKET ");
                }

                return Ok(coffe_amount);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    fn signal_container(&self, lock: &Mutex<Resourse>, cvar: &Condvar, resourse: Resourse) -> Result<(), String> {
        if let Ok(mut old_resourse) = lock.lock() {
            *old_resourse = resourse;
            cvar.notify_all();
            println!("[coffee dispenser] - send new coffee amount request");
            return Ok(());
        };
        Err("[error] - coffee amount monitor failed in coffee dispenser".to_string())
    }


    fn init_containers(&self, dispenser_semaphore: Arc<Semaphore>,coffe_amount_monitor: Arc<(Mutex<Resourse>, Condvar)>) -> Vec<JoinHandle<()>>{
        let mut containers = Vec::with_capacity(CONTAINERS);
        containers.push(thread::spawn(move || {
            let mut coffee_container = CoffeContainer::new(dispenser_semaphore);
            coffee_container.start(coffe_amount_monitor);
        }));

        containers
    }

    fn kill_containers(&self, containers:Vec<JoinHandle<()>>) {
        for c in containers {
            if c.join().is_ok() {
                println!("[global]  - container killed")
            };
        }
    }

    pub fn start(
        &self,
        machine_monitor: Arc<(Mutex<bool>, Condvar)>,
        ticket_monitor: Arc<(Mutex<Ticket>, Condvar)>,
    ) {
        let has_coffe_sem = Arc::new(Semaphore::new(0));
        let coffe_amount_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
        let containers = self.init_containers(has_coffe_sem.clone(), coffe_amount_monitor.clone());

        loop {
            let (lock_ticket, cvar_ticket) = &*ticket_monitor;
            if let Ok(coffe_amount) = self.new_ticket(lock_ticket, cvar_ticket) {
                
                let resourse = Resourse::new(coffe_amount);
                let (lock, cvar) = &*coffe_amount_monitor;
                if self.signal_container(lock, cvar, resourse).is_err(){
                    println!("[coffee dispenser] - ERROR - KILLING THREAD ");
                    break;
                }

                if coffe_amount < 0 {
                    println!("[coffee dispenser] - END ");
                    self.kill_containers(containers);
                    break;
                }

                println!("[coffee dispenser] - waiting for coffee container");
                has_coffe_sem.acquire();
                println!("[coffee dispenser] - semaphore acquired");
                if self.dispense(coffe_amount).is_err() {
                    println!("[coffee dispenser] - ERROR - KILLING THREAD ");
                    break;
                }
            }

            // TODO: wait until all dispensers have read amounts (barrier)
            let (lock, cvar) = &*machine_monitor;
            if self.signal_finish(lock, cvar).is_err() {
                println!("[coffee dispenser] - ERROR - KILLING THREAD ");
            };
        }
    }
}

impl Default for CoffeDispenser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod coffedispenser_test {
    use std::{sync::{Arc, Condvar, Mutex}, thread, time::Duration};

    use std_semaphore::Semaphore;

    use crate::{dipensers::coffe_dispenser::CoffeDispenser, ticket::Ticket, containers::resourse::Resourse};

    #[test]
    fn it_should_dispense_2_sec() {
        let coffe_dispenser = CoffeDispenser::new();
        match coffe_dispenser.dispense(2) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_signal_ready_when_finish() {
        let coffe_dispenser = CoffeDispenser::new();
        let monitor = Arc::new((Mutex::new(false), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffe_dispenser.signal_finish(lock, cvar) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_return_10_when_new_ticket_is_ready() {
        let coffe_dispenser = CoffeDispenser::new();
        let mut ticket = Ticket::new(10);

        ticket.ready();
        let monitor = Arc::new((Mutex::new(ticket), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffe_dispenser.new_ticket(lock, cvar) {
            Ok(coffe_amount) => assert_eq!(coffe_amount, 10),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_signal_when_new_coffe_amount_is_ready() {
        let coffe_dispenser = CoffeDispenser::new();
        let resourse: Resourse = Resourse::new(0);

        
        let monitor = Arc::new((Mutex::new(resourse), Condvar::new()));
        let (lock, cvar) = &*monitor;
        let new_resourse: Resourse = Resourse::new(20);


        match coffe_dispenser.signal_container(lock, cvar,new_resourse) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_continue_when_sem_is_release(){
        let coffe_dispenser = CoffeDispenser::new();
        let sem = Arc::new(Semaphore::new(0));
        let sem_clone = sem.clone();

        thread::sleep(Duration::from_secs(2));
        sem_clone.release();
        sem.acquire();
        match coffe_dispenser.dispense(2) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

}

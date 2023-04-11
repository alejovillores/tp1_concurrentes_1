use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    containers::{coffee_container::CoffeContainer, container::Container, resourse::Resourse},
    helpers::ticket::Ticket,
};

const END: i32 = 0;

#[derive(Debug)]
pub struct CoffeDispenser {}

impl CoffeDispenser {
    pub fn new() -> Self {
        Self {}
    }

    // Simulate dispense time
    fn dispense(&self, amount: i32) -> Result<(), std::fmt::Error> {
        println!("[coffee dispenser] - dispensing {} units of coffee", amount);
        thread::sleep(Duration::from_secs(amount as u64));
        println!("[coffee dispenser] - coffee amount dispensed");
        Ok(())
    }

    // Signals coffee machine that coffee dispenser finished
    fn notify_machine(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<(), String> {
        if let Ok(mut dispenser) = lock.lock() {
            *dispenser = false;
            cvar.notify_all();
            println!("[coffee dispenser] - send signal_finish");
            return Ok(());
        };
        Err("[error] - ticket monitor failed in coffee dispenser".to_string())
    }

    // Waits form a new ticket from coffee machine
    fn wait_new_ticket(&self, lock: &Mutex<Ticket>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut ticket) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let coffe_amount = ticket.get_coffe_amount();
                if coffe_amount >= END {
                    println!("[coffee dispenser] - NEW TICKET ");
                }
                ticket.read();

                return Ok(coffe_amount);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    // Signal coffee container that amount of coffee is needed
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
            println!("[coffee dispenser] - send new coffee amount request");
            return Ok(());
        };
        Err("[error] - coffee amount monitor failed in coffee dispenser".to_string())
    }

    // waits for coffee container to respond
    fn wait_coffe_container(&self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let coffee_amount = resourse.get_amount();
                resourse.read();
                println!("[coffee dispenser] - response from container");
                return Ok(coffee_amount);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    fn init_container(
        &self,
        coffe_response_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        coffe_request_monitor: Arc<(Mutex<Resourse>, Condvar)>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut coffee_container = CoffeContainer::new();
            coffee_container.start(coffe_request_monitor, coffe_response_monitor);
        })
    }

    fn kill_container(&self, container: JoinHandle<()>) {
        if container.join().is_ok() {
            println!("[global]  - coffee container killed")
        };
    }

    pub fn start(
        &self,
        machine_monitor: Arc<(Mutex<bool>, Condvar)>,
        ticket_monitor: Arc<(Mutex<Ticket>, Condvar)>,
    ) {
        let coffe_amount_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
        let coffe_response_monitor = Arc::new((Mutex::new(Resourse::new(0)), Condvar::new()));
        let coffee_container_handler =
            self.init_container(coffe_response_monitor.clone(), coffe_amount_monitor.clone());

        loop {
            let (lock_ticket, cvar_ticket) = &*ticket_monitor;
            if let Ok(mut coffe_amount) = self.wait_new_ticket(lock_ticket, cvar_ticket) {
                let resourse = Resourse::new(coffe_amount);

                let (lock, cvar) = &*coffe_amount_monitor;
                if self.notify_container(lock, cvar, resourse).is_err() {
                    println!("[coffee dispenser] - ERROR - KILLING THREAD ");
                    break;
                }

                println!("[coffee dispenser] - waiting for coffee container");
                let (res_lock, res_cvar) = &*coffe_response_monitor;
                if let Ok(coffee_delivered) = self.wait_coffe_container(res_lock, res_cvar) {
                    coffe_amount = coffee_delivered;
                }
                if coffe_amount < 0 {
                    println!("[coffee dispenser] - END ");
                    self.kill_container(coffee_container_handler);
                    break;
                }
                if self.dispense(coffe_amount).is_err() {
                    println!("[coffee dispenser] - ERROR - KILLING THREAD ");
                    break;
                }
            }

            // TODO: wait until all dispensers have read amounts (barrier)

            let (lock, cvar) = &*machine_monitor;
            if self.notify_machine(lock, cvar).is_err() {
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
    use std::sync::{Arc, Condvar, Mutex};

    use crate::{
        containers::resourse::Resourse, dispensers::coffee_dispenser::CoffeDispenser,
        helpers::ticket::Ticket,
    };

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

        match coffe_dispenser.notify_machine(lock, cvar) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_return_10_when_wait_new_ticket_is_ready() {
        let coffe_dispenser = CoffeDispenser::new();
        let mut ticket = Ticket::new(10);

        ticket.ready_to_read();
        let monitor = Arc::new((Mutex::new(ticket), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffe_dispenser.wait_new_ticket(lock, cvar) {
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

        match coffe_dispenser.notify_container(lock, cvar, new_resourse) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }
}

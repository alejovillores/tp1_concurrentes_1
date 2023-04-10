use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use crate::ticket::Ticket;

const END: i32 = 0;

#[derive(Debug)]
pub struct CoffeDispenser {}

impl CoffeDispenser {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    fn dispense(&self, amount: i32) -> Result<(), std::fmt::Error> {
        // TODO: should signal coffe container that amount of coffe is needed
        // TODO: should wait for coffe container to allow me.

        // imitate dispense
        println!("[coffe dispenser] - dispensing {} units of coffee", amount);
        thread::sleep(Duration::from_secs(amount as u64));
        println!("[coffe dispenser] - coffe amount dispensed");
        Ok(())
    }

    #[allow(dead_code)]
    fn signal_finish(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<(), String> {
        if let Ok(mut dispenser) = lock.lock() {
            *dispenser = false;
            cvar.notify_all();
            println!("[coffe dispenser] - send signal_finish");
            return Ok(());
        };
        Err("[error] - ticket monitor failed in coffee dispenser".to_string())
    }

    #[allow(dead_code)]
    fn new_ticket(&self, lock: &Mutex<Ticket>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            // As long as the value inside the `Mutex<bool>` is `true`, we wait
            if let Ok(ticket) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let coffe_amount = ticket.get_coffe_amount();
                if coffe_amount >= END {
                    println!("[coffe dispenser] - NEW TICKET ");
                }

                return Ok(coffe_amount);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    pub fn start(
        &self,
        machine_monitor: Arc<(Mutex<bool>, Condvar)>,
        ticket_monitor: Arc<(Mutex<Ticket>, Condvar)>,
    ) {
        loop {
            let (lock_ticket, cvar_ticket) = &*ticket_monitor;
            if let Ok(coffe_amount) = self.new_ticket(lock_ticket, cvar_ticket) {
                if coffe_amount < 0 {
                    println!("[coffe dispenser] - END ");
                    break;
                }
                if self.dispense(coffe_amount).is_err() {
                    println!("[coffe dispenser] - ERROR - KILLING THREAD ");
                    break;
                }
            }

            // TODO: wait until all dispensers have read amounts (barrier)
            let (lock, cvar) = &*machine_monitor;
            if self.signal_finish(lock, cvar).is_err() {
                println!("[coffe dispenser] - ERROR - KILLING THREAD ");
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

    use crate::{dipensers::coffe_dispenser::CoffeDispenser, ticket::Ticket};

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
}

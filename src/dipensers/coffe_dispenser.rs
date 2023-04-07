use std::{
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
};

use crate::ticket::Ticket;

#[derive(Debug)]
pub struct CoffeDispenser {}

impl CoffeDispenser {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    fn dispense(&self, amount: i32) -> Result<(), std::fmt::Error> {
        // should signal coffe container that amount of coffe is needed
        // should wait for coffe container to allow me.

        // imitate dispense
        thread::sleep(Duration::from_secs(amount as u64));

        Ok(())
    }

    #[allow(dead_code)]
    fn signal_finish(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<(), String> {
        if let Ok(mut ticket) = lock.lock() {
            *ticket = false;
            cvar.notify_all();
            return Ok(());
        };
        Err("[error] - ticket monitor failed in coffe dispenser".to_string())
    }

    #[allow(dead_code)]
    fn new_ticket(&self, lock: &Mutex<Ticket>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            // As long as the value inside the `Mutex<bool>` is `true`, we wait
            if let Ok(ticket) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                // change flags to true
                let coffe_amount = ticket.get_coffe_amount();
                return Ok(coffe_amount);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    #[allow(dead_code)]
    pub fn start(&self) {

        // wait coffe struct smaphore
        // read coffe amount
        // wait until all dispensers have read amounts

        // dispense

        // change dispenser flag to true for coffemachine
        // signal coffe machine
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

use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use crate::{dispensers::coffee_dispenser::CoffeDispenser, helpers::ticket::Ticket};

const DISPENSERS: usize = 1;

#[derive(Debug)]
pub struct CoffeMachine {
    coffe_made: i32,
}

impl CoffeMachine {
    pub fn new() -> Self {
        let coffe_made = 0;
        Self { coffe_made }
    }

    fn init_dispensers(
        &mut self,
        machine_monitor: Arc<(Mutex<bool>, Condvar)>,
        ticket_monitor: Arc<(Mutex<Ticket>, Condvar)>,
    ) -> Vec<JoinHandle<()>> {
        let mut dispensers = Vec::with_capacity(DISPENSERS);
        dispensers.push(thread::spawn(move || {
            let coffe_dispenser = CoffeDispenser::new();
            coffe_dispenser.start(machine_monitor, ticket_monitor);
        }));

        dispensers
    }

    fn finished(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<bool, String> {
        if let Ok(guard) = lock.lock() {
            // As long as the value inside the `Mutex<bool>` is `true`, we wait
            if let Ok(mut _guard) = cvar.wait_while(guard, |status| *status) {
                *_guard = true;
                return Ok(true);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    fn notify_new_ticket(
        &self,
        lock: &Mutex<Ticket>,
        cvar: &Condvar,
        new_ticket: Ticket,
    ) -> Result<(), String> {
        if let Ok(mut ticket) = lock.lock() {
            *ticket = new_ticket;
            ticket.ready_to_read();
            cvar.notify_all();
            return Ok(());
        };
        Err("[error] - ticket monitor failed".to_string())
    }

    fn read_ticket(&self, i: i32) -> Option<Ticket> {
        if i == 3{
            return Some(Ticket::new(-1));
        }
        Some(Ticket::new(i*10))
    }

    fn kill_dispensers(&self, dispensers: Vec<JoinHandle<()>>) {
        for d in dispensers {
            if d.join().is_ok() {
                println!("[global]  - dispenser killed")
            };
        }
    }

    pub fn start(&mut self) {
        let machine_monitor = Arc::new((Mutex::new(false), Condvar::new()));
        let ticket_monitor = Arc::new((Mutex::new(Ticket::new(0)), Condvar::new()));
        let dispensers = self.init_dispensers(machine_monitor.clone(), ticket_monitor.clone());

        for i in 1..4 {
            let (lock, cvar) = &*machine_monitor;
            match self.finished(lock, cvar) {
                Ok(_) => {
                    println!("[coffe machine] - Coffe Machine is free for new tickets");
                    self.coffe_made += 1;
                }
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
            // TODO: read new line
            // create ticket
            match self.read_ticket(i) {
                Some(ticket) => {
                    let (lock_ticket, cvar_ticket) = &*ticket_monitor;
                    match self.notify_new_ticket(lock_ticket, cvar_ticket, ticket) {
                        Ok(_) => {
                            println!("[coffe machine] - Coffe Machine send {} new ticket", i)
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            break;
                        }
                    }
                }
                None => {
                    println!("[coffe machine] - Coffe Machine finished.");
                    break;
                }
            }
        }
        println!("[coffe machine] - Coffe Machine finished.");
        self.kill_dispensers(dispensers);
    }
}

impl Default for CoffeMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod coffemachine_test {
    use std::sync::{Arc, Condvar, Mutex};

    use crate::{coffee_machine::CoffeMachine, helpers::ticket::Ticket};

    #[test]
    fn it_should_initialize_with_0_coffe_made() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        assert_eq!(coffemachine.coffe_made, 0);
    }

    #[test]
    fn it_should_be_true_when_dispensers_are_free() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let monitor = Arc::new((Mutex::new(false), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffemachine.finished(lock, cvar) {
            Ok(result) => assert!(result),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn it_should_signal_coffe_dispenser() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        let ticket = Ticket::new(0);
        let new_ticket = Ticket::new(10);

        let monitor = Arc::new((Mutex::new(ticket), Condvar::new()));

        let monitor_clone = monitor.clone();
        let (lock, cvar) = &*monitor;
        let (lock_clone, cvar_clone) = &*monitor_clone;

        let _ = coffemachine
            .notify_new_ticket(lock_clone, cvar_clone, new_ticket)
            .unwrap();

        if let Ok(guard) = lock.lock() {
            match cvar.wait_while(guard, |ticket: &mut Ticket| ticket.is_not_ready()) {
                Ok(result) => {
                    assert!(!result.is_not_ready());
                    assert_eq!(result.get_coffe_amount(), 10)
                }
                Err(_) => assert!(false),
            };
        } else {
            assert!(false)
        };
    }

    #[test]
    fn it_should_initilize_dispensers() {
        let mut coffemachine: CoffeMachine = CoffeMachine::new();
        let machine_monitor = Arc::new((Mutex::new(false), Condvar::new()));
        let ticket_monitor = Arc::new((Mutex::new(Ticket::new(0)), Condvar::new()));

        let dispensers = coffemachine.init_dispensers(machine_monitor, ticket_monitor);
        assert_eq!(dispensers.len(), 1)
    }
}

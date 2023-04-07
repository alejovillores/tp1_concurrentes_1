use std::sync::{Arc, Condvar, Mutex};

use crate::ticket::Ticket;

#[derive(Debug)]
pub struct CoffeMachine {
    _coffe_made: i32,
}

impl CoffeMachine {
    pub fn new() -> Self {
        let _coffe_made = 0;
        Self { _coffe_made }
    }

    fn finished(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<bool, String> {
        if let Ok(guard) = lock.lock() {
            // As long as the value inside the `Mutex<bool>` is `true`, we wait

            if let Ok(mut _guard) = cvar.wait_while(guard, |status| *status) {
                // change flags to true
                *_guard = true;
                return Ok(true);
            }
        };
        Err("[error] - machine ready monitor failed".to_string())
    }

    fn notify_new_ticket(&self, lock: &Mutex<Ticket>, cvar: &Condvar) -> Result<(), String> {
        if let Ok(mut ticket) = lock.lock() {
            ticket.ready();
            cvar.notify_all();
            return Ok(());
        };
        Err("[error] - ticket monitor failed".to_string())
    }

    fn read_ticket(&self) -> Option<Ticket> {
        Some(Ticket::new(10))
    }

    #[allow(dead_code)]
    pub fn start(&self) {
        let monitor = Arc::new((Mutex::new(false), Condvar::new()));
        loop {
            let (lock, cvar) = &*monitor;
            match self.finished(lock, cvar) {
                Ok(_) => {
                    println!("[coffe machine] - Coffe Machine is free for new tickets")
                }
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
            // TODO: read new line
            // create ticket
            match self.read_ticket() {
                Some(ticket) => {
                    let ticket_monitor = Arc::new((Mutex::new(ticket), Condvar::new()));
                    let (lock_ticket, cvar_ticket) = &*ticket_monitor;
                    match self.notify_new_ticket(lock_ticket, cvar_ticket) {
                        Ok(_) => {
                            println!("[coffe machine] - Coffe Machine send new ticket")
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

    use crate::{coffemachine::CoffeMachine, ticket::Ticket};

    #[test]
    fn it_should_initialize_with_0_coffe_made() {
        let coffemachine: CoffeMachine = CoffeMachine::new();
        assert_eq!(coffemachine._coffe_made, 0);
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
        let ticket = Ticket::new(10);
        let monitor = Arc::new((Mutex::new(ticket), Condvar::new()));

        let monitor_clone = monitor.clone();
        let (lock, cvar) = &*monitor;
        let (lock_clone, cvar_clone) = &*monitor_clone;

        let _ = coffemachine
            .notify_new_ticket(lock_clone, cvar_clone)
            .unwrap();

        if let Ok(guard) = lock.lock() {
            match cvar.wait_while(guard, |ticket: &mut Ticket| ticket.is_not_ready()) {
                Ok(result) => assert!(!result.is_not_ready()),
                Err(_) => assert!(false),
            };
        } else {
            assert!(false)
        };
    }
}

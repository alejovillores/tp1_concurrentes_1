use std::sync::{Arc, Condvar, Mutex};

use crate::ticket::Ticket;

#[derive(Debug)]
pub struct CoffeMachine {
    _coffe_made: i32,
}

impl CoffeMachine {
    /// Creates a new [`CoffeMachine`].
    pub fn new() -> Self {
        let _coffe_made = 0;
        Self { _coffe_made }
    }

    pub fn finished(&self, lock: &Mutex<bool>, cvar: &Condvar) -> Result<bool, String> {
        if let Ok(guard) = lock.lock() {
            // As long as the value inside the `Mutex<bool>` is `true`, we wait

            if let Ok(mut _guard) = cvar.wait_while(guard, |status| *status) {
                // change flags to true
                *_guard = true;
                return Ok(true);
            }
        };
        Err("[error] - monitor failed".to_string())
    }

    #[allow(dead_code)]
    pub fn start(&self) {
        let monitor = Arc::new((Mutex::new(false), Condvar::new()));

        loop {
            let (lock, cvar) = &*monitor;
            match self.finished(lock, cvar) {
                Ok(_) => {
                    // read new line
                    // create ticket
                    let _ticket = Ticket::new(10);
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }

        // release semaphore

        // send ticket to mutex for dispensers

        // wait until all the dispensers have finished
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

    use crate::coffemachine::CoffeMachine;

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
}

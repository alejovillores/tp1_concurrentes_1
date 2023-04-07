//use crate::dispensers_flag::DispensersFlags;
//use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub struct CoffeMachine {
    coffe_made: i32,
}

impl CoffeMachine {
    /// Creates a new [`CoffeMachine`].
    pub fn new() -> Self {
        let coffe_made = 0;
        Self { coffe_made }
    }

    pub fn coffe_made(&self) -> i32 {
        // let dispensers_monitor = Arc::new((Mutex::new(t),Condvar::new()));
        self.coffe_made
    }
}

impl Default for CoffeMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod coffemachine_test {
    use crate::coffemachine::CoffeMachine;

    #[test]
    fn it_should_initialize_with_0_coffe_made() {
        let coffemachine: CoffeMachine = CoffeMachine::new();

        assert_eq!(coffemachine.coffe_made(), 0);
    }
}

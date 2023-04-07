use std::{thread, time::Duration};

#[derive(Debug)]
pub struct CoffeDispenser {}

impl CoffeDispenser {
    /// Creates a new [`CoffeDispenser`].
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    fn dispense(&self, amount: i32) -> Result<(), std::fmt::Error> {
        // should signal coffe container that amount of coffe is needed
        // should wait for coffe container to allow me.

        // imitate dispense
        thread::sleep(Duration::from_secs(amount as u64));

        // change dispenser flag to true for coffemachine
        // signal coffe machine
        Ok(())
    }
}

impl Default for CoffeDispenser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod coffedispenser_test {
    use crate::dipensers::coffe_dispenser::CoffeDispenser;

    #[test]
    fn it_should_dispense_2_sec() {
        let coffe_dispenser = CoffeDispenser::new();
        match coffe_dispenser.dispense(2) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }
}

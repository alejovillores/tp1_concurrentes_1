#[derive(Debug)]
pub struct Ticket {
    coffe_amount: i32,
    not_ready: bool,
}

impl Ticket {
    pub fn new(coffe_amount: i32) -> Self {
        let not_ready = true;
        Self {
            coffe_amount,
            not_ready,
        }
    }

    pub fn ready_to_read(&mut self) {
        self.not_ready = false
    }

    pub fn read(&mut self) {
        self.not_ready = true
    }

    pub fn is_not_ready(&self) -> bool {
        self.not_ready
    }

    pub fn get_coffe_amount(&self) -> i32 {
        self.coffe_amount
    }
}

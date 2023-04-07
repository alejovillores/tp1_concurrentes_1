#[derive(Debug)]
pub struct Ticket {
    _coffe_amount: i32,
    not_ready: bool,
}

impl Ticket {
    pub fn new(_coffe_amount: i32) -> Self {
        let not_ready = true;
        Self {
            _coffe_amount,
            not_ready,
        }
    }

    pub fn ready(&mut self) {
        self.not_ready = false
    }

    pub fn is_not_ready(&self) -> bool {
        self.not_ready
    }
}

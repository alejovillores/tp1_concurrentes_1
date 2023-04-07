#[derive(Debug)]
pub struct Ticket {
    _coffe_amount: i32,
}

impl Ticket {
    pub fn new(_coffe_amount: i32) -> Self {
        Self { _coffe_amount }
    }
}

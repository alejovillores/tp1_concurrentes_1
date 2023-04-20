const INGREDIENTS: i32 = 3;

#[derive(Debug, Clone, Copy)]
pub struct Ticket {
    coffe_amount: i32,
    water_amount: i32,
    cacao_amount: i32,
    not_ready: bool,
}

impl Ticket {
    pub fn new(coffe_amount: i32, water_amount: i32, cacao_amount: i32) -> Self {
        let not_ready = true;
        Self {
            coffe_amount,
            water_amount,
            cacao_amount,
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

    pub fn get_water_amount(&self) -> i32 {
        self.water_amount
    }

    pub fn get_cacao_amount(&self) -> i32 {
        self.cacao_amount
    }

    pub fn last(&self) -> bool {
        let mut res = 0;
        res += self.cacao_amount;
        res += self.coffe_amount;
        res += self.water_amount;

        res == -INGREDIENTS
    }
}

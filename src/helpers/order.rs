use super::ingredients::Ingredients;

#[derive(Debug, Clone, Copy)]
pub struct Order {
    coffee_amount: i32,
    water_amount: i32,
    cacao_amount: i32,
    milk_amount: i32,
    foam_amount: i32,
    not_ready: bool,
    last_order: bool,
}

impl Order {
    pub fn new(
        coffee_amount: i32,
        water_amount: i32,
        cacao_amount: i32,
        milk_amount: i32,
        foam_amount: i32,
    ) -> Self {
        let not_ready = true;
        let last_order = false;

        Self {
            coffee_amount,
            water_amount,
            cacao_amount,
            milk_amount,
            foam_amount,
            not_ready,
            last_order,
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

    pub fn last(&mut self) {
        self.last_order = true;
    }

    pub fn is_last(&self) -> bool {
        self.last_order
    }

    pub fn get_ingredient_amount(&self, i: Ingredients) -> i32 {
        match i {
            Ingredients::Coffee => self.coffee_amount,
            Ingredients::Cacao => self.cacao_amount,
            Ingredients::Water => self.water_amount,
            Ingredients::Milk => self.milk_amount,
            Ingredients::Foam => self.foam_amount,
            _ => 0,
        }
    }
}

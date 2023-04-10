
pub struct Resourse {
    amount: i32,
    ready: bool
}


impl Resourse {
    
    pub fn new(amount: i32) -> Self {
        let ready = false;
        Self {
            amount,
            ready
        }
    }

    pub fn ready(&mut self) {
        self.ready = true;
    }

    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }
}


#[cfg(test)]
mod resourse_test {
    use super::Resourse;

    #[test]
    fn it_should_not_be_ready(){
        let amount = 10;
        let resourse = Resourse::new(amount);
        assert!(!resourse.ready)
    }

    
    #[test]
    fn it_should_be_ready_after_read_value(){
        let amount = 10;
        let mut resourse = Resourse::new(amount);
        resourse.ready();
        assert!(resourse.ready)
    }

}

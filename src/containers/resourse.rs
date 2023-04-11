pub struct Resourse {
    amount: i32,
    not_ready: bool,
}

impl Resourse {
    pub fn new(amount: i32) -> Self {
        let not_ready = true;
        Self { amount, not_ready }
    }

    pub fn ready_to_read(&mut self) {
        self.not_ready = false;
    }

    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    pub fn is_not_ready(&self) -> bool {
        self.not_ready
    }

    pub fn read(&mut self) {
        self.not_ready = true;
    }
}

#[cfg(test)]
mod resourse_test {
    use super::Resourse;

    #[test]
    fn it_should_not_be_ready() {
        let amount = 10;
        let resourse = Resourse::new(amount);
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_be_ready() {
        let amount = 10;
        let mut resourse = Resourse::new(amount);
        resourse.ready_to_read();
        assert!(!resourse.not_ready)
    }

    #[test]
    fn it_should_be_not_ready_after_read() {
        let amount = 10;
        let mut resourse = Resourse::new(amount);
        resourse.read();
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_have_amount_10() {
        let amount = 10;
        let resourse = Resourse::new(amount);
        assert_eq!(resourse.get_amount(), 10)
    }
}

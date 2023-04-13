#[derive(Debug)]
pub struct Resourse {
    amount: i32,
    dispenser_id: i32,
    not_ready: bool,
}

impl Resourse {
    pub fn new(amount: i32, dispenser_id: i32) -> Self {
        let not_ready = true;
        Self {
            amount,
            dispenser_id,
            not_ready,
        }
    }

    pub fn ready_to_read(&mut self) {
        self.not_ready = false;
    }

    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    pub fn get_dispenser_id(&self) -> i32 {
        self.dispenser_id
    }

    pub fn is_not_ready(&self, dispenser_id: i32) -> bool {
        if !self.not_ready && (dispenser_id == self.dispenser_id) {
            return false;
        }
        true
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
        let resourse = Resourse::new(amount, 1);
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_be_ready() {
        let amount = 10;
        let mut resourse = Resourse::new(amount, 1);
        resourse.ready_to_read();
        assert!(!resourse.not_ready)
    }

    #[test]
    fn it_should_be_not_ready_after_read() {
        let amount = 10;
        let mut resourse = Resourse::new(amount, 1);
        resourse.read();
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_have_amount_10() {
        let amount = 10;
        let resourse = Resourse::new(amount, 1);
        assert_eq!(resourse.get_amount(), 10)
    }

    #[test]
    fn it_shoul_be_for_dispenser_1() {
        let resourse = Resourse::new(1, 1);
        assert_eq!(resourse.get_dispenser_id(), 1)
    }

    #[test]
    fn it_should_be_true_when_not_ready_and_same_id() {
        let resourse = Resourse::new(1, 1);
        assert!(resourse.is_not_ready(1))
    }

    #[test]
    fn it_should_be_false_when_ready_and_same_id() {
        let mut resourse = Resourse::new(1, 1);
        resourse.ready_to_read();
        assert!(!resourse.is_not_ready(1))
    }

    #[test]
    fn it_should_be_true_when_not_ready_and_dif_id() {
        let resourse = Resourse::new(1, 2);
        assert!(resourse.is_not_ready(1))
    }
    #[test]
    fn it_should_be_true_when_ready_and_dif_id() {
        let mut resourse = Resourse::new(1, 1);
        resourse.ready_to_read();
        assert!(resourse.is_not_ready(3))
    }
}

#[derive(Debug)]
pub struct ContainerMessage {
    amount: i32,
    not_ready: bool,
}

impl ContainerMessage {
    pub fn new(amount: i32) -> Self {
        let not_ready = true;
        Self { amount, not_ready }
    }
    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    pub fn ready_to_read(&mut self) {
        self.not_ready = false;
    }

    pub fn is_not_ready(&self) -> bool {
        self.not_ready
    }

    pub fn read(&mut self) {
        self.not_ready = true;
    }
}

#[cfg(test)]
mod container_message_test {

    use super::ContainerMessage;

    #[test]
    fn it_should_not_be_ready() {
        let amount = 10;
        let resourse = ContainerMessage::new(amount);
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_be_ready() {
        let amount = 10;
        let mut resourse = ContainerMessage::new(amount);
        resourse.ready_to_read();
        assert!(!resourse.not_ready)
    }

    #[test]
    fn it_should_be_not_ready_after_read() {
        let amount = 10;
        let mut resourse = ContainerMessage::new(amount);
        resourse.read();
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_have_amount_10() {
        let amount = 10;
        let resourse = ContainerMessage::new(amount);
        assert_eq!(resourse.get_amount(), 10)
    }

    #[test]
    fn it_should_be_true_when_not_ready() {
        let resourse = ContainerMessage::new(1);
        assert!(resourse.is_not_ready())
    }

    #[test]
    fn it_should_be_false_when_ready() {
        let mut resourse = ContainerMessage::new(1);
        resourse.ready_to_read();
        assert!(!resourse.is_not_ready())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ContainerMessageType {
    ResourseRequest,
    DataRequest,
    KillRequest,
}

#[derive(Debug)]
pub struct ContainerMessage {
    message_type: ContainerMessageType,
    amount: i32,
    not_ready: bool,
}

impl ContainerMessage {
    pub fn new(amount: i32, message_type: ContainerMessageType) -> Self {
        let not_ready = true;
        Self {
            message_type,
            amount,
            not_ready,
        }
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

    pub fn get_type(&self) -> ContainerMessageType {
        self.message_type
    }
}

#[cfg(test)]
mod container_message_test {

    use crate::helpers::container_message::ContainerMessageType;

    use super::ContainerMessage;

    #[test]
    fn it_should_not_be_ready() {
        let amount = 10;
        let _type = ContainerMessageType::ResourseRequest;
        let resourse = ContainerMessage::new(amount, _type);
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_be_ready() {
        let amount = 10;
        let _type = ContainerMessageType::ResourseRequest;
        let mut resourse = ContainerMessage::new(amount, _type);
        resourse.ready_to_read();
        assert!(!resourse.not_ready)
    }

    #[test]
    fn it_should_be_not_ready_after_read() {
        let amount = 10;
        let _type = ContainerMessageType::ResourseRequest;
        let mut resourse = ContainerMessage::new(amount, _type);
        resourse.read();
        assert!(resourse.not_ready)
    }

    #[test]
    fn it_should_have_amount_10() {
        let amount = 10;
        let _type = ContainerMessageType::ResourseRequest;
        let resourse = ContainerMessage::new(amount, _type);
        assert_eq!(resourse.get_amount(), 10)
    }

    #[test]
    fn it_should_be_true_when_not_ready() {
        let _type = ContainerMessageType::ResourseRequest;
        let resourse = ContainerMessage::new(1, _type);
        assert!(resourse.is_not_ready())
    }

    #[test]
    fn it_should_be_false_when_ready() {
        let _type = ContainerMessageType::ResourseRequest;
        let mut resourse = ContainerMessage::new(1, _type);
        resourse.ready_to_read();
        assert!(!resourse.is_not_ready())
    }
}

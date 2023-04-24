use super::container::Container;

const CAPACITY: i32 = 1000;
const FINISH_FLAG: i32 = -1;

pub struct MilkContainer {
    capacity: i32,
}

impl MilkContainer {
    pub fn new() -> Self {
        let capacity = CAPACITY;
        Self { capacity }
    }

    fn consume(&mut self, amount: i32) -> Result<i32, String> {
        if (amount.is_positive()) && (amount <= self.capacity) {
            self.capacity -= amount;
            Ok(amount)
        } else {
            Ok(FINISH_FLAG)
        }
    }
}

impl Container for MilkContainer {
    fn start(
        &mut self,
        request_monitor: std::sync::Arc<(
            std::sync::Mutex<crate::helpers::resourse::Resourse>,
            std::sync::Condvar,
        )>,
        response_monitor: std::sync::Arc<(
            std::sync::Mutex<crate::helpers::resourse::Resourse>,
            std::sync::Condvar,
        )>,
        bussy_sem: std::sync::Arc<std_semaphore::Semaphore>,
    ) {
        todo!()
    }
}

#[cfg(test)]
mod milk_container_test {
    use crate::containers::milk_container::{MilkContainer, FINISH_FLAG};

    #[test]
    fn it_should_init_with_n() {
        let n: i32 = 1000;
        let cacao_container = MilkContainer::new();
        assert_eq!(cacao_container.capacity, n)
    }

    #[test]
    fn it_should_consume_when_amount_is_smaller_than_capacity() {
        let mut cacao_container = MilkContainer::new();
        let amount = 10;
        cacao_container.consume(amount).unwrap();
        assert_eq!(cacao_container.capacity, 990)
    }

    #[test]
    fn it_should_send_finish_flag_when_no_capacity() {
        let mut cacao_container = MilkContainer::new();
        let amount = 1100;
        let res = cacao_container.consume(amount).unwrap();
        assert_eq!(res, FINISH_FLAG)
    }
}

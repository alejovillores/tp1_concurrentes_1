const N: i32 = 1000;
const FINISH_FLAG: i32 = -1;

pub struct CacaoContainer {
    capacity: i32,
}

impl CacaoContainer {
    pub fn new() -> Self {
        let capacity = N;
        Self { capacity }
    }

    fn consume(&mut self, amount: i32) -> Result<i32, String> {
        if (amount.is_positive()) && (amount <= self.capacity) {
            self.capacity -= amount;
            return Ok(amount);
        } else {
            return Ok(FINISH_FLAG);
        }
    }
}

#[cfg(test)]
mod cacao_container_test {
    use crate::containers::cacao_container::CacaoContainer;

    #[test]
    fn it_should_init_with_n() {
        let n: i32 = 1000;
        let cacao_container = CacaoContainer::new();
        assert_eq!(cacao_container.capacity, n)
    }

    #[test]
    fn it_should_consume_when_amount_is_smaller_than_capacity() {
        let mut cacao_container = CacaoContainer::new();
        let amount = 10;
        cacao_container.consume(amount).unwrap();
        assert_eq!(cacao_container.capacity, 990)
    }

    #[test]
    fn it_should_return_finish_flag_when_amount_negative() {
        let mut cacao_container = CacaoContainer::new();
        let finish_flag = -1;
        let amount = -1;
        let res = cacao_container.consume(amount).unwrap();
        assert_eq!(res, finish_flag)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CoffeContainer {
    capacity: i32,
}

impl CoffeContainer {
    /// Creates a new [`CoffeContainer`].
    pub fn new(capacity: i32) -> Self {
        Self { capacity }
    }
}

#[cfg(test)]
mod coffecontainer_test {
    use crate::containers::coffe_container::CoffeContainer;

    #[test]
    fn it_should_init_with_30_capacity() {
        let coffe_dispenser = CoffeContainer::new(30);
        assert_eq!(coffe_dispenser.capacity, 30)
    }
}

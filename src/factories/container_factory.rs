use crate::helpers::ingredients::Ingredients;

pub struct ContainerFactory {}

const INGREDIENTS: [Ingredients; 6] = [
    Ingredients::Coffee,
    Ingredients::Milk,
    Ingredients::Water,
    Ingredients::Foam,
    Ingredients::Cacao,
    Ingredients::CoffeGrain,
];

impl ContainerFactory {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_containers() {
        for ingredient in INGREDIENTS.iter().copied() {
            match ingredient {
                Ingredients::Coffee => todo!(),
                Ingredients::CoffeGrain => todo!(),
                Ingredients::Milk => todo!(),
                Ingredients::Foam => todo!(),
                Ingredients::Cacao => todo!(),
                Ingredients::Water => todo!(),
            }
        }
    }
}

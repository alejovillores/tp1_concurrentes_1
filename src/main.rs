use std::env;
use tp1_alejovillores::coffee_machine::CoffeMachine;
const DEFAULT_DISPENSERS: i32 = 1;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);
    match args.len() {
        2 => {
            let path: &String = &args[1].to_string();
            let dispensers: i32 = DEFAULT_DISPENSERS;
            let mut coffe_machine = CoffeMachine::new(path.clone(), dispensers);
            coffe_machine.start();
        }
        3 => {
            let path: &String = &args[1].to_string();
            let dispensers = &args[2].to_owned().parse::<i32>().unwrap();
            let mut coffe_machine = CoffeMachine::new(path.clone(), dispensers.to_owned());
            coffe_machine.start();
        }
        _ => {
            println!(" filename argument must be provided")
        }
    }
}

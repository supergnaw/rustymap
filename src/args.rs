use std::env;

#[derive(Debug)]
pub struct Args {
    pub config_file: String,
}

pub trait ArgParse {
    fn load() -> Self;
}

impl ArgParse for Args {
    fn load() -> Args {
        let arguments: Vec<String> = env::args().collect();

        // prepare struct with default values
        let mut args = Args {
            config_file: String::from("config.toml"),
        };

        // parse command line arguments
        for i in 0..arguments.len() {
            if "--config" == arguments[i] {
                args.config_file = String::from(&arguments[i + 1]);
            }
        }

        println!("successfully loaded arguments: {:?}", &args);

        args
    }
}
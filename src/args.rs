use lexopt;

pub struct Arguments {
    pub data_path: String,
    pub address: String,
    pub noauth: bool,
}

pub fn parse_args() -> Arguments {
    use lexopt::prelude::*;
    let mut parser = lexopt::Parser::from_env();

    match std::env::current_exe() {
        Ok(exec_path) => {
            let mut data_path = exec_path
                .parent()
                .map(|path| path.to_str())
                .flatten()
                .unwrap()
                .to_string()
                + "/kosync-data";
            let exec_name = exec_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap()
                .to_string();
            let mut address: String = "127.0.0.1:8778".to_string();
            let mut noauth = false;

            while let Ok(Some(arg)) = parser.next() {
                match arg {
                    Short('h') | Long("help") => {
                        println!("Usage: {} [-h|--help] [-v|--version] [--data PATH] [--address ADDRESS]", exec_name);
                        println!("\t--data {}\n\t\tThe path to the folder where data will be stored. It should be synced across devices.", data_path);
                        println!("\t--address 127.0.0.1:8778\n\t\tThe address of the api server that kosync-light will serve.");
                        println!("\t--noauth\n\t\tTells the server to ignore all authentication. Data will be stored under \"noauth\".");
                        std::process::exit(0);
                    }
                    Short('v') | Long("version") => {
                        println!("{}", exec_path.to_str().unwrap());
                        println!("v{}", env!("CARGO_PKG_VERSION"));
                        std::process::exit(0);
                    }
                    Long("data") => {
                        data_path = parser
                            .value()
                            .ok()
                            .and_then(|val| val.into_string().ok())
                            .unwrap_or(data_path);
                    }
                    Long("address") => {
                        address = parser
                            .value()
                            .ok()
                            .and_then(|val| val.into_string().ok())
                            .unwrap_or(address);
                    }
                    Long("noauth") => {
                        noauth = true;
                    }
                    _ => continue,
                }
            }
            return Arguments {
                data_path,
                address,
                noauth,
            };
        }
        Err(err) => {
            eprintln!("Failed to get executable path: {}", err);
            std::process::exit(1);
        }
    }
}

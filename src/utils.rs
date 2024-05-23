use std::{env, fs::File, io::Read};

use crate::{Runner, ServerConfig};

pub fn take_args() -> Option<Runner> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match &args[1][..] {
            "--server" => Some(Runner::Server),
            "--client" => Some(Runner::Client),
            _ => None,
        }
    } else {
        None
    }
}

pub fn read_server_config() -> Option<ServerConfig> {
    let mut server_config_file = match File::open("configs/server_config.txt") {
        Ok(server_config_file) => server_config_file,
        Err(_) => return None,
    };
    let mut server_configs = String::new();
    match server_config_file.read_to_string(&mut server_configs) {
        Ok(_) => {
            let server_configs: Vec<String> =
                server_configs.split("\n").map(|x| x.to_string()).collect();
            let server_address = match server_configs[0].split(":").last() {
                Some(server_address_unchecked) => match server_address_unchecked.parse() {
                    Ok(server_address) => server_address,
                    Err(_) => return None,
                },
                None => return None,
            };
            let port = match server_configs[1].split(":").last() {
                Some(port_unchecked) => match port_unchecked.parse() {
                    Ok(port) => port,
                    Err(_) => return None,
                },
                None => return None,
            };
            let difficulty = match server_configs[2].split(":").last() {
                Some(difficulty_unchecked) => match difficulty_unchecked.parse() {
                    Ok(difficulty) => difficulty,
                    Err(_) => return None,
                },
                None => return None,
            };
            Some(ServerConfig {
                server_address,
                port,
                difficulty,
            })
        }
        Err(_) => None,
    }
}

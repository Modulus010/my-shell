use std::env;
use std::io;
use std::io::Write;
use std::process;
use std::process::Command;
use std::process::Stdio;

fn main() {
    let path_home = env::var("USERPROFILE")
        .unwrap_or_else(|_| env::var("HOME").expect("no HOME or USERPROFILE env var"));
    let username = env::var("USERNAME")
        .unwrap_or_else(|_| env::var("USER").expect("no USERNAME or USER env var"));
    let conputername = env::var("COMPUTERNAME")
        .unwrap_or_else(|_| env::var("HOSTNAME").expect("no COMPUTERNAME or HOSTNAME env var"));

    loop {
        print!(
            "{}@{}:{}> ",
            username,
            conputername,
            match env::current_dir().unwrap().file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => env::current_dir().unwrap().to_string_lossy().into_owned(),
            }
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().len() == 0 {
            continue;
        }
        let mut commands = input.trim().split('|').peekable();
        let mut last_child = None;

        while let Some(command) = commands.next() {
            let mut parts = command.split_whitespace();
            let program;
            match parts.next() {
                Some(part) => program = part,
                None => {
                    eprintln!("syntax error near unexpected token `|'");
                    break;
                }
            }
            let mut args = parts.peekable();

            match program {
                "exit" => {
                    return;
                }
                "cd" => {
                    last_child = None;
                    let path = String::from(args.next().unwrap_or("~")).replace("~", &path_home);
                    if args.peek().is_some() {
                        eprintln!("cd: too many arguments");
                        continue;
                    }
                    if let Err(err) = env::set_current_dir(path) {
                        eprintln!("cd: {}", err);
                    }
                }
                program => {
                    let cfg_in = last_child.map_or(Stdio::inherit(), |child: process::Child| {
                        Stdio::from(child.stdout.unwrap())
                    });
                    let cfg_out = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };
                    let child = Command::new(program)
                        .args(args)
                        .stdin(cfg_in)
                        .stdout(cfg_out)
                        .spawn();

                    match child {
                        Ok(child) => {
                            last_child = Some(child);
                        }
                        Err(err) => {
                            last_child = None;
                            eprintln!("{}: {}", program, err);
                        }
                    }
                }
            }
        }
        if let Some(mut child) = last_child {
            child.wait().unwrap();
        }
    }
}

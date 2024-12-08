use std::env;
use std::io;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

fn get_env_var(var_name: &str, fallback: Option<&str>) -> String {
    env::var(var_name).unwrap_or_else(|_| {
        fallback
            .and_then(|f| env::var(f).ok())
            .expect(&format!("no {} or fallback env var", var_name))
    })
}

fn get_current_dir() -> String {
    let current_dir = env::current_dir().unwrap();
    match current_dir.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => current_dir.to_string_lossy().into_owned(),
    }
}

fn handle_cd<'a>(args: &mut impl Iterator<Item = &'a str>, home_path: &str) -> Result<(), String> {
    let path = args.next().unwrap_or("~").replace("~", &home_path);
    if args.next().is_some() {
        return Err(String::from("cd: too many arguments"));
    }
    env::set_current_dir(path).map_err(|err| format!("cd: {}", err))?;
    Ok(())
}

fn main() {
    let home_path = get_env_var("HOME", Some("USERPROFILE"));
    let username = get_env_var("USER", Some("USERNAME"));
    let conputername = get_env_var("HOSTNAME", Some("COMPUTERNAME"));

    loop {
        print!("{}@{}:{}> ", username, conputername, get_current_dir());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            continue;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let mut commands = input.split('|').peekable();
        let mut pipe = Stdio::inherit();
        let mut childs = Vec::new();

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
            let mut args = parts;

            match program {
                "exit" => {
                    return;
                }
                "cd" => {
                    if let Err(err) = handle_cd(&mut args, &home_path) {
                        eprintln!("{}", err);
                    }
                    pipe = Stdio::null();
                }
                program => {
                    let cfg_in = pipe;
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
                        Ok(mut child) => {
                            pipe = child
                                .stdout
                                .take()
                                .map_or_else(|| Stdio::null(), Stdio::from);
                            childs.push(child);
                        }
                        Err(err) => {
                            pipe = Stdio::null();
                            eprintln!("{}: {}", program, err);
                        }
                    }
                }
            }
        }

        for mut child in childs {
            let _ = child.wait();
        }
    }
}

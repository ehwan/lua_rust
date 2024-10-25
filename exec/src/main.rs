use std::io::Write;

use lua_ir::LuaEnv;

fn main() {
    let mut env = LuaEnv::new();
    // env.eval_chunk(b"print('Hello, World!')").unwrap();

    let mut arg = std::env::args();
    arg.next();
    if let Some(filename) = arg.next() {
        let source = std::fs::read_to_string(&filename).expect("failed to read file");
        match env.feed_line(source.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                let message = e.to_error_message(&env);
                eprintln!("{}", message);
            }
        }
        env.clear_feed_pending();
    }

    loop {
        let mut input = String::new();

        if env.is_feed_pending() {
            print!(">> ");
        } else {
            print!("> ");
        }
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();

        match env.feed_line(input.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                let message = e.to_error_message(&env);
                eprintln!("{}", message);
            }
        }
    }
}

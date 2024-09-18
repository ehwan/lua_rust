use codespan_reporting::term;
use codespan_reporting::{
    files::SimpleFiles,
    term::termcolor::{ColorChoice, StandardStream},
};

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let source = std::fs::read_to_string(&filename).expect("failed to read file");

    let block = match lua_parser::parse_str(&source) {
        Ok(block) => block,
        Err(err) => {
            let mut files = SimpleFiles::new();
            let file_id = files.add(&filename, &source);

            let diag = err.to_diag(file_id);
            let writer = StandardStream::stderr(ColorChoice::Auto);
            let config = term::Config::default();
            term::emit(&mut writer.lock(), &config, &files, &diag)
                .expect("Failed to write to stderr");

            return;
        }
    };

    let mut semantics = lua_semantics::Context::new();
    let enhanced = match semantics.process(block) {
        Ok(block) => block,
        Err(err) => {
            let mut files = SimpleFiles::new();
            let file_id = files.add(&filename, &source);

            let diag = err.to_diag(file_id);
            let writer = StandardStream::stderr(ColorChoice::Auto);
            let config = term::Config::default();
            term::emit(&mut writer.lock(), &config, &files, &diag)
                .expect("Failed to write to stderr");

            return;
        }
    };
    println!("{:#?}", enhanced);

    let context = lua_ir::Context::new();
    let program = context.emit(enhanced, semantics);
    let mut stack = program.new_stack();

    loop {
        match program.cycle(&mut stack) {
            Ok(b) => {
                if b {
                    break;
                }
            }
            Err(err) => {
                /*
                let mut files = SimpleFiles::new();
                let file_id = files.add(&filename, &source);

                let diag = err.to_diag(file_id);
                let writer = StandardStream::stderr(ColorChoice::Auto);
                let config = term::Config::default();
                term::emit(&mut writer.lock(), &config, &files, &diag)
                    .expect("Failed to write to stderr");
                */
                println!("{:?}", err);

                return;
            }
        }
    }

    for (i, instr) in program.instructions.iter().enumerate() {
        println!("{:04}: {:?}", i, instr);
    }
}

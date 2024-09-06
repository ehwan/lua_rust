fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let source = std::fs::read_to_string(&filename).expect("failed to read file");

    let block = lua_parser::parse_str(&source).expect("failed to parse");
    println!("{:#?}", block);
}

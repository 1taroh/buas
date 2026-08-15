fn main() -> () {
    let args = std::env::args().skip(1);
    println!("{}", args.collect::<Vec<_>>().join(" "));
}

mod checkpoint;
mod cli;
mod generator;
mod parser;
mod shared;
mod validator;

fn main() -> std::io::Result<()> {
    cli::main()
}

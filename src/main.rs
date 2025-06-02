mod lib;
use lib::*;
use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opt {
    /// Input file
    #[arg(short = 'i', long = "input", value_name = "Input File Path")]
    input: Option<String>,
    #[arg(short = 'o', long = "output", value_name = "Output File Path")]
    output: Option<String>,
    #[arg(long = "stdin", help = "Read from standard input")]
    stdin: bool,
    #[arg(long = "stdout", help = "Write to standard output")]
    stdout: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let opt = Opt::parse();
    let input_md = if opt.stdin {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).expect("Failed to read from stdin");
        buf
    } else {
        let input_path = match &opt.input {
            Some(path) => path,
            None => {
                eprintln!("Error: --input <Input File Path> is required (or use --stdin)");
                std::process::exit(1);
            }
        };
        std::fs::read_to_string(input_path).expect("Failed to read input file")
    };
    let rendered = lib::process_markdown(&input_md);
    if opt.stdout {
        println!("{}", rendered);
    } else {
        let output_path = match &opt.output {
            Some(path) => path,
            None => {
                eprintln!("Error: --output <Output File Path> is required (or use --stdout)");
                std::process::exit(1);
            }
        };
        std::fs::write(output_path, rendered).unwrap();
    }
}

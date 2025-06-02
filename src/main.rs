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
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let opt = Opt::parse();
    let input_path = match &opt.input {
        Some(path) => path,
        None => {
            eprintln!("Error: --input <Input File Path> is required");
            std::process::exit(1);
        }
    };
    let output_path = match &opt.output {
        Some(path) => path,
        None => {
            eprintln!("Error: --output <Output File Path> is required");
            std::process::exit(1);
        }
    };
    // println!("Input: {}", input_path);
    // println!("Output: {}", output_path);
    let input_md = std::fs::read_to_string(input_path).expect("Failed to read input file");
    let rendered = lib::process_markdown(&input_md);
    std::fs::write(output_path, rendered).unwrap();
}

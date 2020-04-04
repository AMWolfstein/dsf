use dsf::DsfFile;
use std::env;
use std::path::Path;

fn main() {
    println!("DSF Probe");

    let clargs = CLArgs::new();
    if clargs.verbose {
        clargs.print();
    }

    if clargs.help {
        clargs.print_usage();
        std::process::exit(0);
    }

    let input_path = Path::new(&clargs.input_filename);
    println!("\nFile = {}", input_path.display());

    match DsfFile::open(input_path) {
        Ok(dsf) => println!("\n{}", dsf),
        Err(_err) => println!("Error parsing DSF file."),
    }
}

struct CLArgs {
    help: bool,
    input_filename: String,
    verbose: bool,
}
impl CLArgs {
    pub fn new() -> CLArgs {
        let mut help = false;
        let mut input_filename = String::from("index.xhtml");
        let mut verbose = false;

        let mut args: Vec<String> = env::args().collect();
        let mut i = 0;

        while i < args.len() {
            match args[i].as_ref() {
                "--help" | "-h" => {
                    help = true;
                }
                "--verbose" | "-v" => {
                    verbose = true;
                }
                _ => {
                    if i == args.len() - 1 {
                        input_filename = args.remove(i);
                    }
                }
            }

            i += 1;
        }

        CLArgs {
            help,
            input_filename,
            verbose,
        }
    }

    fn print(&self) {
        println!("Help: {}", self.help);
        println!("Input filename: {}", self.input_filename);
        println!("Verbose: {}", self.verbose);
        println!("");
    }

    fn print_usage(&self) {
        println!("Usage: dsf-probe [OPTIONS] INPUT_DSF_FILE");
        println!("Options:");
        println!(" -h, --help\t\t\tshow this message and exit");
        println!(" -v, --verbose\t\t\tprint messages");
        println!("\nFor more information see <https://gitlab.com/danieljrmay/dsd>")
    }
}

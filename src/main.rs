use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;
use std::thread;
use structopt::StructOpt;

macro_rules! error {
    ($fmt:tt $($arg:tt)*) => {{
        eprintln!(concat!("error: ", $fmt) $($arg)*);
        process::exit(1)
    }};
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(help = "Input file")]
    input: PathBuf,
    #[structopt(
        long,
        short = "s",
        value_name = "bytes",
        help = "Specify the stack size"
    )]
    stack_size: Option<usize>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let input = fs::read_to_string(&opt.input)?;

    let result = {
        let mut builder = thread::Builder::new().name("runtime".to_owned());
        if let Some(stack_size) = opt.stack_size {
            builder = builder.stack_size(stack_size);
        }
        builder
            .spawn(move || {
                let stdin = io::stdin();
                let stdout = io::stdout();
                let reader = Box::new(stdin);
                let writer = stdout.lock();
                lazylazyk::parse(&input).map(|prog| lazylazyk::run(reader, writer, prog))
            })?
            .join()
            .unwrap()
    };

    result
        .unwrap_or_else(|err| error!("parse failed at line {} column {}", err.line, err.column))?
        .map_or_else(|| error!("attempt to output non-numeral"), process::exit)
}

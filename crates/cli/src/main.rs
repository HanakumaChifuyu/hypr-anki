use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "test_cli", version, about = " 测试clap用法的cli ")]
struct Cli {
    name: String,
    #[command(subcommand)]
    command: Command,
}

/// test_cli add "name"
/// test_cli list --all
#[derive(Parser, Debug)]
enum Command {
    Add {
        name: String,
    },
    List {
        #[arg(short, long)]
        all: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:#?}");
}

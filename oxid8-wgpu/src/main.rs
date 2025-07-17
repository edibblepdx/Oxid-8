use clap::Parser;
use oxid8_wgpu::{Config, run};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(required = true)]
    rom_path: String,
}

impl Into<Config> for Args {
    fn into(self) -> Config {
        Config {
            rom_path: self.rom_path.into(),
        }
    }
}

fn main() {
    let args = Args::parse();
    run(args.into()).unwrap();
}

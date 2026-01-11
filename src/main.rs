mod protocol;
mod sys;

use clap::Parser;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t=String::from("8.8.8.8"))]
    ip: String,

    #[arg(short, long, default_value_t = 4)]
    packet_num: u16,
}

fn main() {
    let args = Args::parse();

    sys::send_icmp_packets(args);
}

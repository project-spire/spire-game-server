mod net;

include!(concat!(env!("OUT_DIR"), "/spire.msg.rs"));

fn main() {
    let heartbeat = Heartbeat {};

    println!("Created heartbeat message!");
}

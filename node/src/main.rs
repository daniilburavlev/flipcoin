use std::process::exit;

use node::run;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Err(e) = run().await {
        eprintln!("{}", e);
        exit(-1);
    }
}

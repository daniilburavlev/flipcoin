use std::{fs::OpenOptions, io::Write, path::Path};

use common::{AppResult, error::AppError};
use wallet::Wallet;

pub(crate) fn keygen(keystore: &Path) -> AppResult<()> {
    let wallet = Wallet::default();
    let secret = wallet.secret();
    let value = serde_json::to_string(&secret).map_err(|_| AppError::Decode)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(keystore)
        .unwrap();
    writeln!(file, "{}", value)?;
    println!("{}", wallet.address_str());
    Ok(())
}

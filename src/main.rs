use std::error::Error;
use tokio::io::AsyncWriteExt;
use ethers::{prelude::*, signers::LocalWallet};
use rand::rngs::OsRng;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    ethereum_node: String,
    bsc_node: String,
    polygon_node: String,
    arbitrum_node: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string("settings.json")?;
    let settings: Settings = serde_json::from_str(&file_contents)?;
    let ethereum_rpc_url: &str = &settings.ethereum_node;
    let ethereum_provider = Arc::new(Provider::<Http>::try_from(ethereum_rpc_url)?);
    let bsc_rpc_url: &str = &settings.bsc_node;
    let bsc_provider = Arc::new(Provider::<Http>::try_from(bsc_rpc_url)?);
    let polygon_rpc_url: &str = &settings.polygon_node;
    let polygon_provider = Arc::new(Provider::<Http>::try_from(polygon_rpc_url)?);
    let arbitrum_rpc_url: &str = &settings.arbitrum_node;
    let arbitrum_provider = Arc::new(Provider::<Http>::try_from(arbitrum_rpc_url)?);

    print!("Введите количество потоков: ");
    io::stdout().flush().unwrap();

    let mut threads = String::new();

    io::stdin().read_line(&mut threads)
        .expect("Ошибка ввода");

    let threads: u128 = threads.trim().parse()
        .expect("Ошибка ввода");

    println!("Starting Work..\n");

    let mut tasks = Vec::new();

    for _ in 0..threads {
        let ethereum_provider_clone = Arc::clone(&ethereum_provider);
        let bsc_provider_clone = Arc::clone(&bsc_provider);
        let polygon_provider_clone = Arc::clone(&polygon_provider);
        let arbitrum_provider_clone = Arc::clone(&arbitrum_provider);

        let handle = tokio::spawn(async move {
            loop {
                let _result = generate_wallet_and_check_balance(&ethereum_provider_clone, &bsc_provider_clone, &polygon_provider_clone, &arbitrum_provider_clone).await;
            }
        });

        tasks.push(handle);
    }

    for handle in tasks {
        handle.await.unwrap();
    }

    Ok(())
}


#[derive(Debug)]
struct CustomError(String);

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for CustomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

async fn write_to_file(filename: &str, text: &str) -> Result<(), Box<dyn Error>> {
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .await
        .map_err(|e| Box::new(CustomError(format!("Failed to open file: {}", e))))?;
    file.write_all(text.as_bytes())
        .await
        .map_err(|e| Box::new(CustomError(format!("Failed to write to file: {}", e))))?;
    file.write_all(b"\n")
        .await
        .map_err(|e| Box::new(CustomError(format!("Failed to write newline to file: {}", e))))?;
    Ok(())
}




async fn generate_wallet_and_check_balance(ethereum_provider: &Provider<Http>, bsc_provider: &Provider<Http>, polygon_provider: &Provider<Http>, arbitrum_provider: &Provider<Http>) -> Result<(), Box<dyn std::error::Error>> {
    let wallet = LocalWallet::new(&mut OsRng);
    let signer = wallet.signer();
    let address = wallet.address();
    let private_key_bytes = signer.to_bytes();
    let private_key_hex = hex::encode(&private_key_bytes);

    let ethereum_balance: U256 = ethereum_provider.get_balance(address, None).await?;
    let ethereum_balance = ethereum_balance.as_u128() as f64 / 1e18;

    let bsc_balance: U256 = bsc_provider.get_balance(address, None).await?;
    let bsc_balance = bsc_balance.as_u128() as f64 / 1e18;

    let polygon_balance: U256 = polygon_provider.get_balance(address, None).await?;
    let polygon_balance = polygon_balance.as_u128() as f64 / 1e18;

    let arbitrum_balance: U256 = arbitrum_provider.get_balance(address, None).await?;
    let arbitrum_balance = arbitrum_balance.as_u128() as f64 / 1e18;

    if ethereum_balance > 0.0 || bsc_balance > 0.0 || polygon_balance > 0.0 || arbitrum_balance > 0.0 {
        println!("Address: {:?}, PrivateKey: {:?}, ETH Balance: {:?}, BSC Balance: {:?}, MATIC Balance: {:?}, ARBITRUM Balance: {:?}", address, private_key_hex, ethereum_balance, bsc_balance, polygon_balance, arbitrum_balance);
        write_to_file("with_balance.txt", format!("Address: {:?}, PrivateKey: {:?}, ETH Balance: {:?}, BSC Balance: {:?}, MATIC Balance: {:?}, ARBITRUM Balance: {:?}", address, private_key_hex, ethereum_balance, bsc_balance, polygon_balance, arbitrum_balance).as_str()).await?;
    }

    Ok(())
}


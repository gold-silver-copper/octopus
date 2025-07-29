use csv::ReaderBuilder;
use serde::Deserialize;
use std::{
    env,
    fs::File,
    io::{self, Write},
};

type ClientID = u16;
type TransactionID = u32;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    client: ClientID,
    tx: TransactionID,
    amount: Option<f64>, // Optional because not all transaction types include amount
}

fn main() {
    // Get the input file path from the first command-line argument
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).unwrap();
    println!("{}", input_path);

    // Write header to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
}

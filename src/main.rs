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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MAKE SURE U LOCK AND OUTPUT ON STDOUT
    // Write header to stdout
    //  let stdout = io::stdout();
    // let mut handle = stdout.lock();

    // Get the input file path from the first command-line argument
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Requires exactly one command line argument. Example: 'cargo run -- test.csv' ");
        std::process::exit(1);
    }

    let path = &args[1];
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new().trim(csv::Trim::All).from_reader(file);

    for result in rdr.deserialize() {
        let record: Transaction = result?;
        println!("{:?}", record);
    }
    Ok(())
}

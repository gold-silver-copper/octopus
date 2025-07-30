use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
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
    amount: Option<Decimal>, // Optional because not all transaction types include amount
}

#[derive(Debug, Serialize)]
struct Account {
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl Account {
    fn new() -> Self {
        Account {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
        }
    }

    fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
    }

    fn withdraw(&mut self, amount: Decimal) {
        if self.locked || self.available < amount {
            return;
        }
        self.available -= amount;
    }

    fn dispute(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    fn resolve(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;

        self.locked = true;
    }
    fn get_total(&self) -> Decimal {
        self.available + self.held
    }
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

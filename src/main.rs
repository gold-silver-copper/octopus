use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
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
#[derive(Debug)]
struct TransactionRecord {
    transaction: Transaction,
    is_disputed: bool,
}

struct TransactionMap(HashMap<TransactionID, TransactionRecord>);
impl TransactionMap {
    fn new() -> TransactionMap {
        TransactionMap(HashMap::new())
    }
}

struct AccountMap(HashMap<ClientID, Account>);

impl AccountMap {
    fn get_or_create_acc(&mut self, cid: &ClientID) -> &mut Account {
        self.0.entry(cid.clone()).or_insert_with(Account::new)
    }
    fn new() -> AccountMap {
        AccountMap(HashMap::new())
    }
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

    let mut account_map = AccountMap::new();
    let mut transaction_map = TransactionMap::new();

    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        let account = account_map.get_or_create_acc(&transaction.client);
        println!("{:?}", transaction);

        match transaction.tx_type {
            TransactionType::Deposit => {
                if let Some(amount) = transaction.amount {
                    account.deposit(amount);
                    transaction_map.0.insert(
                        transaction.tx,
                        TransactionRecord {
                            transaction,
                            is_disputed: false,
                        },
                    );
                }
            }

            TransactionType::Withdrawal => {
                if let Some(amount) = transaction.amount {
                    if account.available >= amount {
                        account.withdraw(amount);
                        // We don’t store withdrawals in the transaction map
                    }
                }
            }

            TransactionType::Dispute => {
                if let Some(record) = transaction_map.0.get_mut(&transaction.tx) {
                    if record.transaction.client == transaction.client && !record.is_disputed {
                        if let Some(amount) = record.transaction.amount {
                            account.dispute(amount);
                            record.is_disputed = true;
                        }
                    }
                }
            }

            TransactionType::Resolve => {
                if let Some(record) = transaction_map.0.get_mut(&transaction.tx) {
                    if record.transaction.client == transaction.client && record.is_disputed {
                        if let Some(amount) = record.transaction.amount {
                            account.resolve(amount);
                            record.is_disputed = false;
                        }
                    }
                }
            }

            TransactionType::Chargeback => {
                if let Some(record) = transaction_map.0.get_mut(&transaction.tx) {
                    if record.transaction.client == transaction.client && record.is_disputed {
                        if let Some(amount) = record.transaction.amount {
                            account.chargeback(amount);
                            record.is_disputed = false;
                        }
                    }
                }
            }
        }
    }
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["client", "available", "held", "total", "locked"])?;
    for (client_id, acc) in account_map.0.iter() {
        wtr.write_record(&[
            client_id.to_string(),
            acc.available.to_string(),
            acc.held.to_string(),
            acc.get_total().to_string(),
            acc.locked.to_string(),
        ])?;
    }
    wtr.flush()?;

    Ok(())
}

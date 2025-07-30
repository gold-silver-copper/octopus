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

type TransactionMap = HashMap<TransactionID, TransactionRecord>;

type AccountMap = HashMap<ClientID, Account>;

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
        if self.locked {
            eprintln!("Attempted deposit of {} on locked account", amount);
            return;
        }
        self.available += amount;
    }

    fn withdraw(&mut self, amount: Decimal) {
        if self.locked {
            eprintln!("Attempted withdrawal of {} on locked account", amount);
            return;
        }
        if self.available < amount {
            eprintln!(
                "Insufficient funds: attempted withdrawal of {}, available is {}",
                amount, self.available
            );
            return;
        }
        self.available -= amount;
    }

    fn dispute(&mut self, amount: Decimal) {
        if self.locked {
            eprintln!("Attempted dispute of {} on locked account", amount);
            return;
        }
        if self.available < amount {
            eprintln!(
                "Dispute failed: amount {} exceeds available funds {}",
                amount, self.available
            );
            return;
        }
        self.available -= amount;
        self.held += amount;
    }

    fn resolve(&mut self, amount: Decimal) {
        if self.locked {
            eprintln!("Attempted resolve of {} on locked account", amount);
            return;
        }
        if self.held < amount {
            eprintln!(
                "Resolve failed: amount {} exceeds held funds {}",
                amount, self.held
            );
            return;
        }
        self.held -= amount;
        self.available += amount;
    }

    fn chargeback(&mut self, amount: Decimal) {
        if self.locked {
            eprintln!(
                "Attempted chargeback of {} on already locked account",
                amount
            );
            return;
        }
        if self.held < amount {
            eprintln!(
                "Chargeback failed: amount {} exceeds held funds {}",
                amount, self.held
            );
            return;
        }
        self.held -= amount;
        self.locked = true;
    }

    fn get_total(&self) -> Decimal {
        self.available + self.held
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        // Turn csv entry into rust struct for manipulation
        let transaction: Transaction = result?;
        //Get an account reference by client_id. If account does not exist, create a new one and return reference to new account.
        let account = account_map
            .entry(transaction.client)
            .or_insert_with(Account::new);

        match transaction.tx_type {
            TransactionType::Deposit => match transaction.amount {
                Some(amount) => {
                    account.deposit(amount);
                    transaction_map.insert(
                        transaction.tx,
                        TransactionRecord {
                            transaction,
                            is_disputed: false,
                        },
                    );
                }
                None => {
                    eprintln!("Deposit transaction {} missing amount", transaction.tx);
                }
            },

            TransactionType::Withdrawal => {
                match transaction.amount {
                    Some(amount) => {
                        account.withdraw(amount);
                        // We don’t store withdrawals in the transaction map
                    }
                    None => {
                        eprintln!("Withdraw transaction {} missing amount", transaction.tx);
                    }
                }
            }

            //Here we make sure that the transaction and dispute client is the same, and that the transaction is not already disputed.
            TransactionType::Dispute => match transaction_map.get_mut(&transaction.tx) {
                Some(record)
                    if record.transaction.client == transaction.client && !record.is_disputed =>
                {
                    match record.transaction.amount {
                        Some(amount) => {
                            account.dispute(amount);
                            record.is_disputed = true;
                        }
                        None => {
                            eprintln!("Dispute transaction {} missing amount", transaction.tx);
                        }
                    }
                }
                Some(_) => {
                    eprintln!(
                        "Dispute transaction {} has client mismatch or is already disputed",
                        transaction.tx
                    );
                }
                None => {
                    eprintln!("Dispute transaction {} not found", transaction.tx);
                }
            },

            TransactionType::Resolve => match transaction_map.get_mut(&transaction.tx) {
                Some(record)
                    if record.transaction.client == transaction.client && record.is_disputed =>
                {
                    match record.transaction.amount {
                        Some(amount) => {
                            account.resolve(amount);
                            record.is_disputed = false;
                        }
                        None => {
                            eprintln!("Resolve transaction {} missing amount", transaction.tx);
                        }
                    }
                }
                Some(_) => {
                    eprintln!(
                        "Resolve transaction {} has client mismatch or is not under dispute",
                        transaction.tx
                    );
                }
                None => {
                    eprintln!("Resolve transaction {} not found", transaction.tx);
                }
            },

            TransactionType::Chargeback => match transaction_map.get_mut(&transaction.tx) {
                Some(record)
                    if record.transaction.client == transaction.client && record.is_disputed =>
                {
                    match record.transaction.amount {
                        Some(amount) => {
                            account.chargeback(amount);
                            record.is_disputed = false;
                        }
                        None => {
                            eprintln!(
                                "Chargeback failed: transaction {} has no amount",
                                transaction.tx
                            );
                        }
                    }
                }
                Some(_) => {
                    eprintln!(
                        "Chargeback failed: transaction {} is not under dispute or has client mismatch",
                        transaction.tx
                    );
                }
                None => {
                    eprintln!(
                        "Chargeback failed: transaction {} not found",
                        transaction.tx
                    );
                }
            },
        }
    }
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["client", "available", "held", "total", "locked"])?;
    for (client_id, acc) in account_map.iter() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::*;

    #[test]
    fn test_deposit_increases_available_and_total() {
        let mut acc = Account::new();
        acc.deposit(dec!(10.5));
        assert_eq!(acc.available, dec!(10.5));
        assert_eq!(acc.get_total(), dec!(10.5));
    }

    #[test]
    fn test_withdraw_succeeds_when_sufficient_funds() {
        let mut acc = Account::new();
        acc.deposit(dec!(10.0));
        acc.withdraw(dec!(4.0));
        assert_eq!(acc.available, dec!(6.0));
        assert_eq!(acc.get_total(), dec!(6.0));
    }

    #[test]
    fn test_withdraw_does_nothing_if_insufficient_funds() {
        let mut acc = Account::new();
        acc.deposit(dec!(5.0));
        acc.withdraw(dec!(10.0));
        assert_eq!(acc.available, dec!(5.0));
        assert_eq!(acc.get_total(), dec!(5.0));
    }

    #[test]
    fn test_withdraw_does_nothing_if_account_locked() {
        let mut acc = Account::new();
        acc.deposit(dec!(5.0));
        acc.locked = true;
        acc.withdraw(dec!(2.0));
        assert_eq!(acc.available, dec!(5.0));
    }

    #[test]
    fn test_dispute_moves_funds_from_available_to_held() {
        let mut acc = Account::new();
        acc.deposit(dec!(10.0));
        acc.dispute(dec!(4.0));
        assert_eq!(acc.available, dec!(6.0));
        assert_eq!(acc.held, dec!(4.0));
        assert_eq!(acc.get_total(), dec!(10.0));
    }

    #[test]
    fn test_resolve_returns_held_to_available() {
        let mut acc = Account::new();
        acc.deposit(dec!(10.0));
        acc.dispute(dec!(3.0));
        acc.resolve(dec!(3.0));
        assert_eq!(acc.available, dec!(10.0));
        assert_eq!(acc.held, dec!(0.0));
    }

    #[test]
    fn test_chargeback_removes_held_and_locks_account() {
        let mut acc = Account::new();
        acc.deposit(dec!(10.0));
        acc.dispute(dec!(7.0));
        acc.chargeback(dec!(7.0));
        assert_eq!(acc.held, dec!(0.0));
        assert_eq!(acc.available, dec!(3.0));
        assert_eq!(acc.get_total(), dec!(3.0));
        assert!(acc.locked);
    }

    #[test]
    fn test_total_is_sum_of_available_and_held() {
        let mut acc = Account::new();
        acc.deposit(dec!(10.0));
        acc.dispute(dec!(4.0));
        assert_eq!(acc.get_total(), dec!(10.0));
    }
}

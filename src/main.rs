use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the input file path from the first command-line argument
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Requires exactly one command line argument. Example: 'cargo run -- test.csv' ");
        std::process::exit(1);
    }

    // We get the second arg here because the first arg is always the destination folder for compilation
    let path = &args[1];
    let file = File::open(path)?;
    //trims whitespace and header
    let mut rdr = ReaderBuilder::new().trim(csv::Trim::All).from_reader(file);

    let mut db = Database::default();

    for result in rdr.deserialize() {
        // Turn csv entry into rust struct for manipulation
        let transaction: Transaction = result?;
        // Process the transaction, save it in the transaction map if it is a deposit or withdrawal.
        db.process(transaction);
    }
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["client", "available", "held", "total", "locked"])?;
    for (client_id, acc) in db.account_map.iter() {
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

type ClientID = u16;
type TransactionID = u32;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Default)]
struct Database {
    transaction_map: TransactionMap,
    account_map: AccountMap,
}
type TransactionMap = HashMap<TransactionID, TransactionRecord>;
type AccountMap = HashMap<ClientID, Account>;

impl Database {
    fn handle_amount_transaction(
        &mut self,
        transaction: &Transaction,
        action: impl Fn(&mut Account, Decimal) -> AccountResult,
    ) {
        let account = self
            .account_map
            .entry(transaction.client)
            .or_insert_with(Account::new);
        if let Some(amount) = transaction.amount {
            if amount < Decimal::ZERO {
                eprintln!(
                    "Amount cannot be negative: transaction ID {} ",
                    transaction.tx
                );
                return;
            }
            if self.transaction_map.contains_key(&transaction.tx) {
                eprintln!(
                    "Duplicate {:#?} transaction ID {}: skipping",
                    transaction.tx_type, transaction.tx
                );
                return;
            }
            match action(account, amount) {
                Ok(()) => {
                    self.transaction_map.insert(
                        transaction.tx,
                        TransactionRecord {
                            transaction: transaction.clone(),
                            is_disputed: false,
                        },
                    );
                }
                Err(err) => {
                    eprintln!(
                        "{:#?} failed for client {} tx {}: {:?}",
                        transaction.tx_type, transaction.client, transaction.tx, err
                    );
                }
            }
        } else {
            eprintln!(
                "{:#?} transaction {} missing amount",
                transaction.tx_type, transaction.tx
            );
        }
    }
    fn handle_dispute_like(
        &mut self,
        transaction: &Transaction,

        condition: impl Fn(&TransactionRecord) -> bool,
        action: impl Fn(&mut Account, Decimal) -> AccountResult,

        new_disputed_state: bool,
    ) {
        let account = self
            .account_map
            .entry(transaction.client)
            .or_insert_with(Account::new);
        match self.transaction_map.get_mut(&transaction.tx) {
            Some(record)
                if record.transaction.client == transaction.client
                    && record.transaction.tx_type == TransactionType::Deposit
                    && condition(record) =>
            {
                if let Some(amount) = record.transaction.amount {
                    match action(account, amount) {
                        Ok(()) => record.is_disputed = new_disputed_state,
                        Err(err) => eprintln!(
                            "{:#?} failed for client {} tx {}: {:?}",
                            transaction.tx_type, transaction.client, transaction.tx, err
                        ),
                    }
                } else {
                    eprintln!(
                        "{:#?} transaction {} missing amount",
                        transaction.tx_type, transaction.tx
                    );
                }
            }
            Some(_) => eprintln!(
                "{:#?} transaction {} has client mismatch, invalid dispute state, or is not a deposit",
                transaction.tx_type, transaction.tx
            ),
            None => eprintln!(
                "{:#?} transaction {} not found",
                transaction.tx_type, transaction.tx
            ),
        }
    }

    fn process(&mut self, transaction: Transaction) {
        match transaction.tx_type {
            TransactionType::Deposit => {
                self.handle_amount_transaction(&transaction, Account::deposit);
            }
            TransactionType::Withdrawal => {
                self.handle_amount_transaction(&transaction, Account::withdraw);
            }
            TransactionType::Dispute => {
                self.handle_dispute_like(
                    &transaction,
                    |record| !record.is_disputed,
                    Account::dispute,
                    true,
                );
            }
            TransactionType::Resolve => {
                self.handle_dispute_like(
                    &transaction,
                    |record| record.is_disputed,
                    Account::resolve,
                    false,
                );
            }
            TransactionType::Chargeback => {
                self.handle_dispute_like(
                    &transaction,
                    |record| record.is_disputed,
                    Account::chargeback,
                    false,
                );
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct Account {
    available: Decimal,
    held: Decimal,
    locked: bool,
}

#[derive(Debug)]
pub enum AccountError {
    Locked,
    InsufficientFunds,
}
pub type AccountResult = Result<(), AccountError>;

impl Account {
    fn new() -> Self {
        Account {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
        }
    }

    fn deposit(&mut self, amount: Decimal) -> AccountResult {
        if self.locked {
            return Err(AccountError::Locked);
        }
        self.available += amount;
        Ok(())
    }

    fn withdraw(&mut self, amount: Decimal) -> AccountResult {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if self.available < amount {
            return Err(AccountError::InsufficientFunds);
        }
        self.available -= amount;
        Ok(())
    }

    fn dispute(&mut self, amount: Decimal) -> AccountResult {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if self.available < amount {
            return Err(AccountError::InsufficientFunds);
        }
        self.available -= amount;
        self.held += amount;
        Ok(())
    }

    fn resolve(&mut self, amount: Decimal) -> AccountResult {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if self.held < amount {
            return Err(AccountError::InsufficientFunds);
        }
        self.held -= amount;
        self.available += amount;
        Ok(())
    }

    fn chargeback(&mut self, amount: Decimal) -> AccountResult {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if self.held < amount {
            return Err(AccountError::InsufficientFunds);
        }
        self.held -= amount;
        self.locked = true;
        Ok(())
    }

    fn get_total(&self) -> Decimal {
        self.available + self.held
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::*;

    fn setup_deposit_transaction(
        tx: TransactionID,
        client: ClientID,
        amount: Decimal,
    ) -> Transaction {
        Transaction {
            tx_type: TransactionType::Deposit,
            client,
            tx,
            amount: Some(amount),
        }
    }

    fn setup_dispute_transaction(tx: TransactionID, client: ClientID) -> Transaction {
        Transaction {
            tx_type: TransactionType::Dispute,
            client,
            tx,
            amount: None,
        }
    }

    #[test]
    fn test_deposit_increases_available_balance() {
        let mut db = Database::default();
        let tx = setup_deposit_transaction(1, 1, dec!(100.00));
        db.process(tx);

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(100.00));
        assert_eq!(acc.held, dec!(0.00));
        assert_eq!(acc.locked, false);
    }

    #[test]
    fn test_withdrawal_reduces_balance() {
        let mut db = Database::default();
        db.process(setup_deposit_transaction(1, 1, dec!(100.00)));

        db.process(Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(dec!(30.00)),
        });

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(70.00));
        assert_eq!(acc.get_total(), dec!(70.00));
    }

    #[test]
    fn test_withdrawal_insufficient_funds_does_not_change_balance() {
        let mut db = Database::default();
        db.process(setup_deposit_transaction(1, 1, dec!(50.00)));

        db.process(Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(dec!(100.00)),
        });

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(50.00)); // unchanged
    }

    #[test]
    fn test_dispute_moves_funds_to_held() {
        let mut db = Database::default();
        db.process(setup_deposit_transaction(1, 1, dec!(100.00)));
        db.process(setup_dispute_transaction(1, 1));

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(0.00));
        assert_eq!(acc.held, dec!(100.00));
    }

    #[test]
    fn test_resolve_returns_held_funds_to_available() {
        let mut db = Database::default();
        db.process(setup_deposit_transaction(1, 1, dec!(100.00)));
        db.process(setup_dispute_transaction(1, 1));

        db.process(Transaction {
            tx_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
        });

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(100.00));
        assert_eq!(acc.held, dec!(0.00));
    }

    #[test]
    fn test_chargeback_removes_held_funds_and_locks_account() {
        let mut db = Database::default();
        db.process(setup_deposit_transaction(1, 1, dec!(100.00)));
        db.process(setup_dispute_transaction(1, 1));

        db.process(Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        });

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(0.00));
        assert_eq!(acc.held, dec!(0.00));
        assert_eq!(acc.locked, true);
    }

    #[test]
    fn test_cannot_deposit_to_locked_account() {
        let mut db = Database::default();
        db.process(setup_deposit_transaction(1, 1, dec!(100.00)));
        db.process(setup_dispute_transaction(1, 1));
        db.process(Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        });

        db.process(setup_deposit_transaction(2, 1, dec!(50.00)));

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(0.00)); // deposit rejected
    }

    #[test]
    fn test_cannot_withdraw_from_locked_account() {
        let mut db = Database::default();
        db.process(setup_deposit_transaction(1, 1, dec!(100.00)));
        db.process(setup_dispute_transaction(1, 1));
        db.process(Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        });

        db.process(Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(dec!(50.00)),
        });

        let acc = db.account_map.get(&1).unwrap();
        assert_eq!(acc.available, dec!(0.00)); // withdrawal ignored
    }

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

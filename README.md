Assumptions: When an account is locked, ALL transactions are blocked, including withdrawals.

# Correctness, Safety, and Performance

Striving for correctness by utilizing the typesystem (newtypes for all uses of u16,u32,hashmaps,etc), using match statements instead of if-else to guarantee handling of all cases, verification against test data sets (test.csv). CSV types are cast to Rust types for extra type checking (Transaction struct). Errors are being handled by logging to stderr. Regression prevented by the use of unit tests.

The rust csv reader does not load the whole csv file into memory at once, instead it reads line by line using a buffer. From the csv documentation: Note that the CSV reader is buffered automatically, so you should not wrap rdr in a buffered reader like io::BufReader.
( https://docs.rs/csv/latest/csv/struct.ReaderBuilder.html )

In a real world scenario where transactions are coming from thousands of concurrent TCP streams, it would be proper to use an async framework like 'tokio' , or a multithreading framework like rayon or crossbeam, or a combination of them. Furthermore, in a real world scenario with servers, one should account for phenomena such as backpressure, DoS attacks, priority(trades from authenticated users vs data query from unauthenticated), and load balancing. I have some experience with this from writing multiplayer game servers.

Rust's borrow checker is helpful here for making sure that we are not mutating two records at the same time, especially in a concurrent scenario. We can take advantage of rust features such as Arc and Mutex, which allow us to safely mutate data across threads, or use specialized crates such as DashMap. In this test example, where each transaction only ever touches one client, we can shard our processing over 'N' servers by keying each shard to client by 'client_id % N'. Shards can be dynamically expanded using a ring buffer (complicated but effective!).

For the accounts, total is not tracked, only held and available are tracked, total is calculated through a function call. This helps lower memory usage in the database, and prevent synchronization issues (accidentally modifying held or available without modifying total).

# AI Disclosure

1. It's been a while since I did anything CLI related so I used AI to generate me skeleton structure of a program that accepts command line arguments. My first guess would have been to use the 'clap' library which I was familiar with, but AI pushed me to use 'std::env::args' instead, since it is zero dependency and sufficient for this task.

2. I then used AI to generate me some code using the csv line by line reader for a sample of the input file. AI created a 'Transaction' struct using String for the transaction type and raw integers for the IDs. To better take advantage of the Rust type system, I created an Enum for the transaction types, and newtypes for the IDs. I have used the 'csv' library many times before in the past, but I like having AI generate me the boilerplate, easier than modifying code from my other projects. AI also helped to properly use the 'serde' decorators on the Enum.

3. I got curious and asked the AI if it had seen similar questions before and it said yes and offered to show me a full solution. Curiousity got the best of me and I said yes. The solution it gave me didn't completely work but had nice helper functions and program structure, it had also incorporated my previous contributions. It also used the Decimal crate instead of floats, which I was planning on researching since the instructions mentioned a decimal accuracy to 4 digits.

4. I spent some time incorporating the AI code into my project, optimizing and adjusting as needed, as well as creating other newtypes that I wanted such as the AccountMap. At some point I had a small double mut borrow error, I asked AI to fix it and it suggested using the entry+or_insert_with function on Hashmap, which I liked and incorporated because it was cleaner than my solution.

5. Once I had written the rest of the code I wanted to, I used AI to populate my  match transaction.tx_type {} statement, which it did quite well. It also generated the final csv output code unprompted.

6. Went through the code cleaning and optimizing it, adding comments, and verifying validity. Then used AI to write unit tests.

7. Used AI to turn all if let blocks into match statements for better error catching. Used AI to add eprintln everywhere it was needed.

8. Moved loop out of main into a separate function. Cleaned up main in general.

9. Put AccountMap and TransactionMap into Database, cleaned up code further.

10. Created AccountError and AccountResult with the help of AI, fixed bug related to adding unsuccesful deposits to the database.

11. After noticing a lot of code duplication in my solution, I used AI to deduplicate by generating functions that utilize closures.

# Random Notes

-Handle disputing a failed withdrawal (AI says: Withdrawals are final by design — you can't freeze or reverse money that's already left. In real world requires separate legal processes)
-Handle cases where the dispute,resolve,or chargeback amount is more than is available/held
-Are deposits,chargebacks, and disputes allowed on a locked account?
-LOG errors in case of invalid transactions
-Need more clarification in terms of which transactions are allowed or disallowed when an account is locked.

Improvements: Have seperate tx_id for disputes that mark the dispute itself, not just a reference tx_id. Do much stricter data verification during parsing, make sure all values have max 4 places past decimal, make sure there are no negative values in amount,

1. It's been a while since I did anything CLI related so I used AI to generate me skeleton structure of a program that accepts command line arguments. My first guess would have been to use the 'clap' library which I was familiar with, but AI pushed me to use 'std::env::args' instead, since it is zero dependency and sufficient for this task.

2. I then used AI to generate me some code using the csv line by line reader for a sample of the input file. AI created a 'Transaction' struct using String for the transaction type and raw integers for the IDs. To better take advantage of the Rust type system, I created an Enum for the transaction types, and newtypes for the IDs. I have used the 'csv' library many times before in the past, but I like having AI generate me the boilerplate, easier than modifying code from my other projects. AI also helped to properly use the 'serde' decorators on the Enum.

3. I got curious and asked the AI if it had seen similar questions before and it said yes and offered to show me a full solution. Curiousity got the best of me and I said yes. The solution it gave me didn't completely work but had nice helper functions and program structure, it had also incorporated my previous contributions. It also used the Decimal crate instead of floats, which I was planning on researching since the instructions mentioned a decimal accuracy to 4 digits.

4. I spent some time incorporating the AI code into my project, optimizing and adjusting as needed, as well as creating other newtypes that I wanted such as the AccountMap. At some point I had a small double mut borrow error, I asked AI to fix it and it suggested using the entry+or_insert_with function on Hashmap, which I liked and incorporated because it was cleaner than my solution.

5. Once I had written the rest of the code I wanted to, I used AI to populate my  match transaction.tx_type {} statement, which it did quite well. It also generated the final csv output code unprompted.

6. Went through the code cleaning and optimizing it, adding comments. Then used AI to write unit tests.

7. Used AI to turn all if let blocks into match statements for better error catching. Used AI to add eprintln everywhere it was needed.

8. Moved loop out of main into a separate function. Cleaned up main in general.

I hope that my extensive use of AI is not off putting, I am trying to be completely honest about the process. I believe that I would have been able to solve the tech assessment even without using AI, but it is a convenient time-saving tool in my programming toolbox.


-Handle disputing a failed withdrawal (AI says: Withdrawals are final by design — you can't freeze or reverse money that's already left. In real world requires separate legal processes)
-Handle cases where the dispute,resolve,or chargeback amount is more than is available/held
-Are deposits,chargebacks, and disputes allowed on a locked account?
-LOG errors in case of invalid transactions
-Need more clarification in terms of which transactions are allowed or disallowed when an account is locked.

Improvements: Have seperate tx_id for disputes that mark the dispute itself, not just a reference tx_id

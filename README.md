1. It's been a while since I did anything CLI related so I asked AI to generate me skeleton structure of a program that accepts command line arguments. My first guess would have been to use the 'clap' library which I was familiar with, but AI pushed me to use 'std::env::args' instead, since it is zero dependency and sufficient for this task.

2. I then asked AI to generate me some code using the csv line by line reader for a sample of the input file. AI created a 'Transaction' struct using String for the transaction type and raw integers for the IDs. To better take advantage of the Rust type system, I created an Enum for the transaction types, and newtypes for the IDs. I have used the 'csv' library many times before in the past, but I like having AI generate me the boilerplate, easier than modifying code from my other projects. AI also helped to properly use the 'serde' decorators on the Enum.

3. I got curious and asked the AI if it had seen similar questions before and it said yes and offered to show me a full solution. Curiousity got the best of me and I said yes. The solution it gave me didn't completely work but had nice helper functions and program structure, it had also incorporated my previous contributions. It also used the Decimal crate instead of floats, which I was planning on researching since the instructions mentioned a decimal accuracy to 4 digits.


I hope that my extensive use of AI is not off putting, I am trying to be completely honest about the process. I believe that I would have been able to solve the tech assessment even without using AI, but it is a convenient time-saving tool in my programming toolbox.


-Handle disputing a failed withdrawal
-Handle cases where the dispute,resolve,or chargeback amount is more than is available/held
-Are deposits,chargebacks, and disputes allowed on a locked account?

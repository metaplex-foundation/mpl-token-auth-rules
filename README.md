# Token Authorization Rules
A program that provides the ability to create and execute rules to restrict common token operations such as transferring and selling.

## Overview
Authorization rules are variants of a `Rule` enum that implements a `validate()` function.

There are **Primitive Rules** and **Composed Rules** that are created by combining of one or more primitive rules.

**Primitive Rules** store any accounts or data needed for evaluation, and at runtime will produce a true or false output based on accounts and a well-defined `Payload` that are passed into the `validate()` function.

**Composed Rules** return a true or false based on whether any or all of the primitive rules return true.  Composed rules can then be combined into higher-level composed rules that implement more complex boolean logic.  Because of the recursive definition of the `Rule` enum, calling `validate()` on a top-level composed rule will start at the top and validate at every level, down to the component primitive rules.

# Examples
## Rust
```rust
use rmp_serde::Serializer;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Signer,
    signer::keypair::Keypair, transaction::Transaction,
};
use token_authorization_rules::{
    state::{Operation, Rule, RuleSet},
    Payload, PayloadVec,
};

fn main() {
    let url = "http://localhost:8899".to_string();
    let rpc_client = RpcClient::new(url);

    let payer = Keypair::new();

    let _signature = rpc_client
        .request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL)
        .unwrap();

    let balance = rpc_client
        .wait_for_balance_with_commitment(
            &payer.pubkey(),
            Some(LAMPORTS_PER_SOL),
            CommitmentConfig::default(),
        )
        .unwrap();

    println!("Payer balance: {}", balance);

    // Find RuleSet PDA.
    let (ruleset_addr, _ruleset_bump) = token_authorization_rules::pda::find_ruleset_address(
        payer.pubkey(),
        "da rulez".to_string(),
    );

    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: payer.pubkey(),
    };
    let adtl_signer2 = Rule::AdditionalSigner {
        account: payer.pubkey(),
    };
    let amount_check = Rule::Amount { amount: 2 };

    let first_rule = Rule::All {
        rules: vec![adtl_signer, adtl_signer2],
    };

    let overall_rule = Rule::All {
        rules: vec![first_rule, amount_check],
    };

    // Create a RuleSet.
    let mut rule_set = RuleSet::new();
    rule_set.add(Operation::Transfer, overall_rule);

    println!("{:#?}", rule_set);

    // Serialize the RuleSet using RMP serde.
    let mut serialized_data = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_data))
        .unwrap();

    // Create a `create` instruction.
    let create_ix = token_authorization_rules::instruction::create(
        token_authorization_rules::id(),
        payer.pubkey(),
        ruleset_addr,
        "da rulez".to_string(),
        serialized_data,
    );

    // Add it to a transaction.
    let latest_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
        &[&payer],
        latest_blockhash,
    );

    // Send and confirm transaction.
    let signature = rpc_client.send_and_confirm_transaction(&create_tx).unwrap();

    println!("Create tx signature: {}", signature);

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::new(None, None, Some(2), None);

    // Create a `validate` instruction.
    let validate_ix = token_authorization_rules::instruction::validate(
        token_authorization_rules::id(),
        payer.pubkey(),
        ruleset_addr,
        "da rulez".to_string(),
        Operation::Transfer,
        payload,
        vec![],
        vec![],
    );

    // Add it to a transaction.
    let latest_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let validate_tx = Transaction::new_signed_with_payer(
        &[validate_ix],
        Some(&payer.pubkey()),
        &[&payer],
        latest_blockhash,
    );

    // Send and confirm transaction.
    let signature = rpc_client
        .send_and_confirm_transaction(&validate_tx)
        .unwrap();

    println!("Validate tx signature: {}", signature);
}
```

## JS
**Coming soon!**

### Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool
3. Run `yarn install` to install dependencies

---

### Build the rust program alone
```
$ yarn build:rust
```

---

### Generate the JS SDK and rebuild IDL only (using shank and solita)
```
$ yarn solita
```

---

### Build the JS SDK only (must be generated first)
```
$ yarn build:sdk
```

---

### Build the program and generate/build the IDL/SDK/docs
```
$ yarn build
```

---

### Start Amman and run the test script
Run the following command in a separate shell
```
$ amman start
```

Then, run the Amman script
```
$ yarn amman
```

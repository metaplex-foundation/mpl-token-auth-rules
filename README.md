# Metaplex Token Authorization Rules
A program that provides the ability to create and execute rules to restrict common token operations such as transferring and selling.

> ⚠️ **Metaplex Token Authorization Rules is currently experimental and has not been formally audited. Use in production at your own risk.**

## Overview
Authorization rules are variants of a `Rule` enum that implements a `validate()` function.

There are **Primitive Rules** and **Composed Rules** that are created by combining of one or more primitive rules.

**Primitive Rules** store any accounts or data needed for evaluation, and at runtime will produce a true or false output based on accounts and a well-defined `Payload` that are passed into the `validate()` function.

**Composed Rules** return a true or false based on whether any or all of the primitive rules return true.  Composed rules can then be combined into higher-level composed rules that implement more complex boolean logic.  Because of the recursive definition of the `Rule` enum, calling `validate()` on a top-level composed rule will start at the top and validate at every level, down to the component primitive rules.

# Examples
## Rust
```rust
use mpl_token_auth_rules::{
    state::{Operation, Rule, RuleSet},
    Payload,
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Signer, signer::keypair::Keypair,
    transaction::Transaction,
};

fn main() {
    let url = "https://api.devnet.solana.com".to_string();
    let rpc_client = RpcClient::new(url);

    let payer = Keypair::new();

    let signature = rpc_client
        .request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL)
        .unwrap();

    loop {
        let confirmed = rpc_client.confirm_transaction(&signature).unwrap();
        if confirmed {
            break;
        }
    }

    // Find RuleSet PDA.
    let (ruleset_addr, _ruleset_bump) =
        mpl_token_auth_rules::pda::find_ruleset_address(payer.pubkey(), "test ruleset".to_string());

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
    let create_ix = mpl_token_auth_rules::instruction::create(
        mpl_token_auth_rules::id(),
        payer.pubkey(),
        ruleset_addr,
        "test ruleset".to_string(),
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
    let validate_ix = mpl_token_auth_rules::instruction::validate(
        mpl_token_auth_rules::id(),
        payer.pubkey(),
        ruleset_addr,
        "test ruleset".to_string(),
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

### Build and test the Rust program
```
$ cd program/
$ cargo build-bpf
$ cargo test-bpf
$ cd ..
```

---

### Build the program, generate the JS API, and rebuild IDL (using Shank and Solita)
```
$ yarn build:rust
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

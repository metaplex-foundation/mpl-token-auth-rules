# Token Authorization Rules
A program that provides the ability to create and execute rules to restrict common token operations such as transferring and selling.

## Overview
Authorization rules are variants of a `Rule` enum that implements a `validation` function.

There are **Primitive Rules** and **Composed Rules** that are created by combining of one or more primitive rules.

**Primitive Rules** store any accounts needed for evaluation, and at runtime will produce a true or false output based on a `HashMap` of accounts passed into the `validate` function.

**Composed Rules** return a true or false based on whether any or all of the primitive rules return true.  Composed rules can then be combined into higher-level composed rules that implement more complex boolean logic.  Because of the recursive definition of the `Rule` enum, calling `validate()` on a top-level composed rule will start at the top and validate at every level, down to the component primitive rules.

# Examples
```
WIP
```

# Template info below
This repo is still WIP.

## Includes Shank/Solita SDK generation, Amman support, scripts, .github configuration, and more!

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

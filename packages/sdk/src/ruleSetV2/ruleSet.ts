import * as beet from '@metaplex-foundation/beet';
import { PublicKey } from '@solana/web3.js';
import { RuleSetRevisionV1, RuleV1 } from '../ruleSetV1';
import { additionalSignerV2 } from './additionalSigner';
import { allV2 } from './all';
import { amountV2 } from './amount';
import { anyV2 } from './any';
import { Base58PublicKey, toBase58PublicKey } from './base58PublicKey';
import { deserializePublicKey, serializePublicKey } from './helpers';
import { namespaceV2 } from './namespace';
import { notV2 } from './not';
import { passV2 } from './pass';
import { pdaMatchV2 } from './pdaMatch';
import { programOwnedV2 } from './programOwned';
import { programOwnedListV2 } from './programOwnedList';
import { programOwnedTreeV2 } from './programOwnedTree';
import { pubkeyListMatchV2 } from './pubkeyListMatch';
import { pubkeyMatchV2 } from './pubkeyMatch';
import { pubkeyTreeMatchV2 } from './pubkeyTreeMatch';
import { RuleV2, deserializeRulesV2, serializeRulesV2 } from './rule';

export type RuleSetRevisionV2 = {
  libVersion: 2;
  name: string;
  owner: Base58PublicKey;
  operations: Record<string, RuleV2>;
};

export const serializeRuleSetRevisionV2 = (ruleSet: RuleSetRevisionV2): Buffer => {
  const tuples = Object.entries(ruleSet.operations);
  const operations = tuples.map(([operation]) => operation);
  const rules = tuples.map(([, rule]) => rule);
  const ruleSize = operations.length;

  // Header.
  const headerBuffer = Buffer.alloc(8);
  beet.u32.write(headerBuffer, 0, 2);
  beet.u32.write(headerBuffer, 4, ruleSize);

  // Owner.
  const ownerBuffer = serializePublicKey(ruleSet.owner);

  // Name.
  const nameBuffer = Buffer.alloc(32);
  nameBuffer.write(ruleSet.name);

  // Operations.
  const operationsBuffer = Buffer.concat(
    operations.map((operation) => {
      const buffer = Buffer.alloc(32);
      buffer.write(operation);
      return buffer;
    }),
  );

  // Rules.
  const rulesBuffer = serializeRulesV2(rules);

  return Buffer.concat([headerBuffer, ownerBuffer, nameBuffer, operationsBuffer, rulesBuffer]);
};

export const deserializeRuleSetRevisionV2 = (buffer: Buffer, offset = 0): RuleSetRevisionV2 => {
  const libVersion = beet.u32.read(buffer, offset);
  offset += 4;
  if (libVersion !== 2) {
    throw new Error('Expected a rule set version 2, got version ' + libVersion);
  }

  // Rule size.
  const ruleSize = beet.u32.read(buffer, offset);
  offset += 4;

  // Owner.
  const owner = deserializePublicKey(buffer, offset);
  offset += 32;

  // Name.
  const name = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  offset += 32;

  // Operations.
  const operations: string[] = [];
  for (let index = 0; index < ruleSize; index++) {
    operations.push(
      buffer
        .subarray(offset, offset + 32)
        .toString()
        .replace(/\u0000/g, ''),
    );
    offset += 32;
  }

  // Rules.
  const rules = deserializeRulesV2(buffer, ruleSize, offset);
  const tuples: [string, RuleV2][] = operations.map((operation, index) => [
    operation,
    rules[index],
  ]);

  return { libVersion: 2, name, owner, operations: Object.fromEntries(tuples) };
};

export const getRuleSetRevisionV2FromV1 = (ruleSetV1: RuleSetRevisionV1): RuleSetRevisionV2 => {
  return {
    libVersion: 2,
    name: ruleSetV1.ruleSetName,
    owner: toBase58PublicKey(new PublicKey(ruleSetV1.owner)),
    operations: Object.fromEntries(
      Object.entries(ruleSetV1.operations).map(([operation, rule]) => [
        operation,
        getRuleV2FromV1(rule),
      ]),
    ),
  };
};

export const getRuleV2FromV1 = (ruleV1: RuleV1): RuleV2 => {
  if (ruleV1 === 'Namespace') {
    return namespaceV2();
  }
  if (ruleV1 === 'Pass') {
    return passV2();
  }
  if ('AdditionalSigner' in ruleV1) {
    return additionalSignerV2(new PublicKey(ruleV1.AdditionalSigner.account));
  }
  if ('All' in ruleV1) {
    return allV2(ruleV1.All.rules.map(getRuleV2FromV1));
  }
  if ('Amount' in ruleV1) {
    return amountV2(ruleV1.Amount.field, ruleV1.Amount.operator, ruleV1.Amount.amount);
  }
  if ('Any' in ruleV1) {
    return anyV2(ruleV1.Any.rules.map(getRuleV2FromV1));
  }
  if ('Not' in ruleV1) {
    return notV2(getRuleV2FromV1(ruleV1.Not.rule));
  }
  if ('PDAMatch' in ruleV1) {
    return pdaMatchV2(
      ruleV1.PDAMatch.pda_field,
      new PublicKey(ruleV1.PDAMatch.program),
      ruleV1.PDAMatch.seeds_field,
    );
  }
  if ('ProgramOwned' in ruleV1) {
    return programOwnedV2(ruleV1.ProgramOwned.field, new PublicKey(ruleV1.ProgramOwned.program));
  }
  if ('ProgramOwnedList' in ruleV1) {
    return programOwnedListV2(
      ruleV1.ProgramOwnedList.field,
      ruleV1.ProgramOwnedList.programs.map((p) => new PublicKey(p)),
    );
  }
  if ('ProgramOwnedTree' in ruleV1) {
    return programOwnedTreeV2(
      ruleV1.ProgramOwnedTree.pubkey_field,
      ruleV1.ProgramOwnedTree.proof_field,
      new Uint8Array(ruleV1.ProgramOwnedTree.root),
    );
  }
  if ('PubkeyListMatch' in ruleV1) {
    return pubkeyListMatchV2(
      ruleV1.PubkeyListMatch.field,
      ruleV1.PubkeyListMatch.pubkeys.map((p) => new PublicKey(p)),
    );
  }
  if ('PubkeyMatch' in ruleV1) {
    return pubkeyMatchV2(ruleV1.PubkeyMatch.field, new PublicKey(ruleV1.PubkeyMatch.pubkey));
  }
  if ('PubkeyTreeMatch' in ruleV1) {
    return pubkeyTreeMatchV2(
      ruleV1.PubkeyTreeMatch.pubkey_field,
      ruleV1.PubkeyTreeMatch.proof_field,
      new Uint8Array(ruleV1.PubkeyTreeMatch.root),
    );
  }
  throw new Error('Unknown rule: ' + JSON.stringify(ruleV1));
};

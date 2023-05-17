import {
  Context,
  PublicKeyBase58,
  Serializer,
  base58,
  base58PublicKey,
  mergeBytes,
  publicKey,
} from '@metaplex-foundation/umi';
import { RuleSetRevisionV1, RuleV1 } from '../v1';
import { additionalSignerV2 } from './additionalSigner';
import { allV2 } from './all';
import { amountV2 } from './amount';
import { anyV2 } from './any';
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
import { RuleV2, getRuleV2Serializer } from './rule';

export type RuleSetRevisionV2 = {
  libVersion: 2;
  name: string;
  owner: PublicKeyBase58;
  operations: Record<string, RuleV2>;
};

export const getRuleSetRevisionV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<RuleSetRevisionV2> => {
  const s = context.serializer;
  return {
    description: 'RuleSetRevisionV2',
    fixedSize: null,
    maxSize: null,
    serialize: (revision: RuleSetRevisionV2) => {
      const tuples = Object.entries(revision.operations);
      const operations = tuples.map(([operation]) => operation);
      const rules = tuples.map(([, rule]) => rule);
      const ruleSize = operations.length;

      return mergeBytes([
        s.u32().serialize(2), // libVersion (0-3)
        s.u32().serialize(ruleSize), // ruleSize (4-7)
        s.string({ encoding: base58, size: 32 }).serialize(revision.owner), // owner (8-39)
        s.string({ size: 32 }).serialize(revision.name), // name (40-71)
        s
          .array(s.string({ size: 32 }), { size: ruleSize })
          .serialize(operations), // operations (72-...)
        s
          .array(getRuleV2Serializer(context), { size: ruleSize })
          .serialize(rules), // rules
      ]);
    },
    deserialize: (buffer: Uint8Array, offset = 0) => {
      const [libVersion, versionOffset] = s.u32().deserialize(buffer, offset);
      offset = versionOffset;
      if (libVersion !== 2) {
        throw new Error(
          `Expected a rule set version 2, got version ${libVersion}`
        );
      }
      const [ruleSize, ruleSizeOffset] = s.u32().deserialize(buffer, offset);
      offset = ruleSizeOffset;
      const [owner, ownerOffset] = s
        .string({ encoding: base58, size: 32 })
        .deserialize(buffer, offset);
      offset = ownerOffset;
      const [name, nameOffset] = s
        .string({ size: 32 })
        .deserialize(buffer, offset);
      offset = nameOffset;
      const [operations, operationsOffset] = s
        .array(s.string({ size: 32 }), { size: ruleSize })
        .deserialize(buffer, offset);
      offset = operationsOffset;
      const [rules, rulesOffset] = s
        .array(getRuleV2Serializer(context), { size: ruleSize })
        .deserialize(buffer, offset);
      offset = rulesOffset;
      const tuples: [string, RuleV2][] = operations.map((operation, index) => [
        operation,
        rules[index],
      ]);
      return [
        { libVersion: 2, name, owner, operations: Object.fromEntries(tuples) },
        offset,
      ];
    },
  };
};

export const getRuleSetRevisionV2FromV1 = (
  ruleSetV1: RuleSetRevisionV1
): RuleSetRevisionV2 => ({
  libVersion: 2,
  name: ruleSetV1.ruleSetName,
  owner: base58PublicKey(new Uint8Array(ruleSetV1.owner)),
  operations: Object.fromEntries(
    Object.entries(ruleSetV1.operations).map(([operation, rule]) => [
      operation,
      getRuleV2FromV1(rule),
    ])
  ),
});

export const getRuleV2FromV1 = (ruleV1: RuleV1): RuleV2 => {
  const toPublicKey = (bytes: number[]) => publicKey(new Uint8Array(bytes));
  if (ruleV1 === 'Namespace') {
    return namespaceV2();
  }
  if (ruleV1 === 'Pass') {
    return passV2();
  }
  if ('AdditionalSigner' in ruleV1) {
    return additionalSignerV2(toPublicKey(ruleV1.AdditionalSigner.account));
  }
  if ('All' in ruleV1) {
    return allV2(ruleV1.All.rules.map(getRuleV2FromV1));
  }
  if ('Amount' in ruleV1) {
    return amountV2(
      ruleV1.Amount.field,
      ruleV1.Amount.operator,
      ruleV1.Amount.amount
    );
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
      toPublicKey(ruleV1.PDAMatch.program),
      ruleV1.PDAMatch.seeds_field
    );
  }
  if ('ProgramOwned' in ruleV1) {
    return programOwnedV2(
      ruleV1.ProgramOwned.field,
      toPublicKey(ruleV1.ProgramOwned.program)
    );
  }
  if ('ProgramOwnedList' in ruleV1) {
    return programOwnedListV2(
      ruleV1.ProgramOwnedList.field,
      ruleV1.ProgramOwnedList.programs.map((p) => toPublicKey(p))
    );
  }
  if ('ProgramOwnedTree' in ruleV1) {
    return programOwnedTreeV2(
      ruleV1.ProgramOwnedTree.pubkey_field,
      ruleV1.ProgramOwnedTree.proof_field,
      new Uint8Array(ruleV1.ProgramOwnedTree.root)
    );
  }
  if ('PubkeyListMatch' in ruleV1) {
    return pubkeyListMatchV2(
      ruleV1.PubkeyListMatch.field,
      ruleV1.PubkeyListMatch.pubkeys.map((p) => toPublicKey(p))
    );
  }
  if ('PubkeyMatch' in ruleV1) {
    return pubkeyMatchV2(
      ruleV1.PubkeyMatch.field,
      toPublicKey(ruleV1.PubkeyMatch.pubkey)
    );
  }
  if ('PubkeyTreeMatch' in ruleV1) {
    return pubkeyTreeMatchV2(
      ruleV1.PubkeyTreeMatch.pubkey_field,
      ruleV1.PubkeyTreeMatch.proof_field,
      new Uint8Array(ruleV1.PubkeyTreeMatch.root)
    );
  }
  throw new Error(`Unknown rule: ${JSON.stringify(ruleV1)}`);
};

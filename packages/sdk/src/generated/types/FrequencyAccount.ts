/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import { Key, keyBeet } from './Key';
export type FrequencyAccount = {
  key: Key;
  lastUpdate: beet.bignum;
  period: beet.bignum;
};

/**
 * @category userTypes
 * @category generated
 */
export const frequencyAccountBeet = new beet.BeetArgsStruct<FrequencyAccount>(
  [
    ['key', keyBeet],
    ['lastUpdate', beet.i64],
    ['period', beet.i64],
  ],
  'FrequencyAccount',
);
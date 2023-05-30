import {
  Context,
  Signer,
  TransactionBuilderGroup,
  transactionBuilder,
  transactionBuilderGroup,
} from '@metaplex-foundation/umi';
import { findRuleSetBufferPda, writeToBufferV1 } from './generated';
import { RuleSetRevision, getRuleSetRevisionSerializer } from './revisions';

export type WriteRuleSetToBufferV1 = {
  /** Payer and creator of the RuleSet buffer. */
  payer?: Signer;
  /** The new revision to write to the buffer. */
  ruleSetRevision: RuleSetRevision;
  /**
   * The size of each chunk to write to the buffer.
   * @default `900`
   */
  chunkSize?: number;
};

export const writeRuleSetToBufferV1 = (
  context: Pick<
    Context,
    'eddsa' | 'programs' | 'serializer' | 'payer' | 'transactions'
  >,
  input: WriteRuleSetToBufferV1
): TransactionBuilderGroup => {
  const payer = input.payer ?? context.payer;
  const chunkSize = input.chunkSize ?? 900;
  const bufferPda = findRuleSetBufferPda(context, { owner: payer.publicKey });
  const serializedRevision = getRuleSetRevisionSerializer(context).serialize(
    input.ruleSetRevision
  );

  const bufferSize = serializedRevision.length;
  const numberOfWrites = Math.ceil(bufferSize / chunkSize);
  const writeInstructions = Array.from(
    { length: numberOfWrites },
    (_, index) => {
      const slice = serializedRevision.slice(
        index * chunkSize,
        Math.min((index + 1) * chunkSize, serializedRevision.length)
      );
      return writeToBufferV1(context, {
        payer,
        bufferPda,
        data: slice,
        overwrite: index === 0,
      });
    }
  );

  return transactionBuilderGroup(
    transactionBuilder()
      .add(writeInstructions)
      .unsafeSplitByTransactionSize(context)
  ).sequential();
};

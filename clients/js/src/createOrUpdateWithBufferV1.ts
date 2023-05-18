import {
  Context,
  Signer,
  TransactionBuilderGroup,
  none,
  transactionBuilder,
  transactionBuilderGroup,
} from '@metaplex-foundation/umi';
import {
  createOrUpdateV1,
  findRuleSetBufferPda,
  findRuleSetPda,
  puffRuleSetV1,
  writeToBufferV1,
} from './generated';
import { RuleSetRevision, getRuleSetRevisionSerializer } from './revisions';

export const PUFF_CHUNK_SIZE = 10_000;

export type CreateOrUpdateWithBufferV1Input = {
  /** Payer and creator of the RuleSet */
  payer?: Signer;
  /** The name of the RuleSet account. */
  ruleSetName: string;
  /** The new revision to add to the RuleSet account. */
  ruleSetRevision: RuleSetRevision;
  /**
   * The size of each chunk to write to the buffer.
   * @default `900`
   */
  chunkSize?: number;
};

export const createOrUpdateWithBufferV1 = (
  context: Pick<
    Context,
    'eddsa' | 'programs' | 'serializer' | 'payer' | 'transactions'
  >,
  input: CreateOrUpdateWithBufferV1Input
): TransactionBuilderGroup => {
  const payer = input.payer ?? context.payer;
  const chunkSize = input.chunkSize ?? 900;
  const bufferPda = findRuleSetBufferPda(context, { owner: payer.publicKey });
  const ruleSetPda = findRuleSetPda(context, {
    owner: payer.publicKey,
    name: input.ruleSetName,
  });
  const serializedRevision = getRuleSetRevisionSerializer(context).serialize(
    input.ruleSetRevision
  );

  // Write instructions.
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

  // Puff instructions.
  const puffSize = serializedRevision.length;
  const numberOfPuffs = Math.ceil(puffSize / PUFF_CHUNK_SIZE) - 1;
  const puffInstructions = Array.from({ length: numberOfPuffs }, () =>
    puffRuleSetV1(context, {
      payer,
      ruleSetPda,
      ruleSetName: input.ruleSetName,
    })
  );

  // Create or update from buffer.
  const builder = transactionBuilder()
    .add(writeInstructions)
    .add(puffInstructions)
    .add(
      createOrUpdateV1(context, {
        payer,
        ruleSetPda,
        bufferPda,
        ruleSetRevision: none(),
      })
    );

  return transactionBuilderGroup(
    builder.unsafeSplitByTransactionSize(context)
  ).sequential();
};

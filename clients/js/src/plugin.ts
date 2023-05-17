import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMplTokenAuthRulesProgram } from './generated';

export const mplTokenAuthRules = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplTokenAuthRulesProgram(), false);
  },
});

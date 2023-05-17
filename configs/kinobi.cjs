const path = require('path');
const k = require('@metaplex-foundation/kinobi');

// Paths.
const clientDir = path.join(__dirname, '..', 'clients');
const idlDir = path.join(__dirname, '..', 'idls');

// Instanciate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, 'mpl_token_auth_rules.json')]);

// Add RuleSet types and account.
kinobi.update(
  new k.TransformNodesVisitor([
    {
      selector: { kind: 'programNode', name: 'mplTokenAuthRules' },
      transformer: (node) => {
        k.assertProgramNode(node);
        return k.programNode({
          ...node,
          accounts: [
            ...node.accounts,
            k.accountNode({
              name: 'ruleSet',
              data: k.accountDataNode({
                name: 'ruleSetAccountData',
                struct: k.structTypeNode([
                  k.structFieldTypeNode({
                    name: 'key',
                    child: k.linkTypeNode('Key'),
                    defaultsTo: {
                      strategy: 'omitted',
                      value: k.vEnum('Key', 'RuleSet'),
                    },
                  }),
                  k.structFieldTypeNode({
                    name: 'revisionMapLocation',
                    child: k.numberTypeNode('u64'),
                  }),
                  k.structFieldTypeNode({
                    name: 'revisions',
                    child: k.bytesTypeNode(k.remainderSize()),
                  }),
                ]),
                link: k.linkTypeNode('ruleSetAccountData', {
                  importFrom: 'hooked',
                }),
              }),
              seeds: [
                k.stringConstantSeed('rule_set'),
                k.publicKeySeed('owner', 'The owner of the rule set.'),
                k.stringSeed('name', 'The name of the rule set.'),
              ],
            }),
            k.accountNode({
              name: 'ruleSetBuffer',
              data: k.accountDataNode({
                name: 'ruleSetBufferAccountData',
                struct: k.structTypeNode([
                  k.structFieldTypeNode({
                    name: 'serializedRuleSet',
                    child: k.bytesTypeNode(k.remainderSize()),
                  }),
                ]),
              }),
              seeds: [
                k.stringConstantSeed('rule_set'),
                k.publicKeySeed('owner', 'The owner of the rule set.'),
              ],
            }),
          ],
        });
      },
    },
    {
      selector: {
        kind: 'structFieldTypeNode',
        name: 'serializedRuleSet',
        stack: ['createOrUpdateArgs'],
      },
      transformer: (node) => {
        return k.structFieldTypeNode({
          ...node,
          name: 'ruleSetRevision',
          child: k.linkTypeNode('ruleSetRevisionInput', {
            importFrom: 'hooked',
          }),
          defaultsTo: { strategy: 'optional', value: k.vNone() },
        });
      },
    },
    {
      selector: {
        kind: 'structFieldTypeNode',
        name: 'serializedRuleSet',
        stack: ['writeToBufferArgs'],
      },
      transformer: (node) => {
        return k.structFieldTypeNode({
          ...node,
          name: 'data',
          child: k.bytesTypeNode(k.prefixedSize(k.numberTypeNode('u32'))),
        });
      },
    },
  ]),
);

// Split instructions.
kinobi.update(
  new k.CreateSubInstructionsFromEnumArgsVisitor({
    createOrUpdate: 'createOrUpdateArgs',
    puffRuleSet: 'puffRuleSetArgs',
    validate: 'validateArgs',
    writeToBuffer: 'writeToBufferArgs',
  }),
);

// Render JavaScript.
const jsDir = path.join(clientDir, 'js', 'src', 'generated');
const prettier = require(path.join(clientDir, 'js', '.prettierrc.json'));
kinobi.accept(new k.RenderJavaScriptVisitor(jsDir, { prettier }));

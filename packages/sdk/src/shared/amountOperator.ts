export enum AmountOperator {
  Lt, // Less Than
  LtEq, // Less Than or Equal To
  Eq, // Equal To
  GtEq, // Greater Than or Equal To
  Gt, // Greater Than
}

export type AmountOperatorString = '<' | '<=' | '=' | '>=' | '>';

export const parseAmountOperator = (
  operator: AmountOperator | AmountOperatorString,
): AmountOperator => {
  return (
    {
      '<': AmountOperator.Lt,
      '<=': AmountOperator.LtEq,
      '=': AmountOperator.Eq,
      '>=': AmountOperator.GtEq,
      '>': AmountOperator.Gt,
    }[operator] ?? operator
  );
};

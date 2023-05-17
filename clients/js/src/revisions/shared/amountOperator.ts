export enum AmountOperator {
  Lt, // Less Than
  LtEq, // Less Than or Equal To
  Eq, // Equal To
  GtEq, // Greater Than or Equal To
  Gt, // Greater Than
}

export type AmountOperatorString = '<' | '<=' | '=' | '>=' | '>';

export const toAmountOperator = (
  operator: AmountOperator | AmountOperatorString
): AmountOperator =>
  (<Record<AmountOperator | AmountOperatorString, AmountOperator>>{
    '<': AmountOperator.Lt,
    '<=': AmountOperator.LtEq,
    '=': AmountOperator.Eq,
    '>=': AmountOperator.GtEq,
    '>': AmountOperator.Gt,
  })[operator] ?? operator;

export const toAmountOperatorString = (
  operator: AmountOperator | AmountOperatorString
): AmountOperatorString =>
  (<Record<AmountOperator | AmountOperatorString, AmountOperatorString>>{
    [AmountOperator.Lt]: '<',
    [AmountOperator.LtEq]: '<=',
    [AmountOperator.Eq]: '=',
    [AmountOperator.GtEq]: '>=',
    [AmountOperator.Gt]: '>',
  })[operator] ?? operator;

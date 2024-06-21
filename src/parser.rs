/// Recursive Descent Parser.
use crate::tokenizer::Tokenizer;

struct Parser {
    string: String,
    tokenizer: Tokenizer,
    lookahead: Option<Literal>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub literal_type: LiteralType,
    pub value: Box<LiteralValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    Type(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Value(String),
    NestedValue(Option<Literal>),
    NestedValueList(Vec<Option<Literal>>),
}

impl Parser {
    fn new() -> Self {
        Self {
            string: String::new(),
            tokenizer: Tokenizer::new(String::new()),
            lookahead: None,
        }
    }

    /// Parses a string into an AST.
    fn parse(&mut self, string: String) -> Option<Literal> {
        self.string = string.clone();
        self.tokenizer.string = LiteralType::Type(string.clone());

        // Prime the tokenizer to obtain the first token
        // which is our lookahead for predictive parsing.
        self.lookahead = self.tokenizer.get_next_token();

        self.program()
    }

    /// Main Entry Point
    ///
    /// Program
    ///   : NumericLiteral
    ///   ;
    fn program(&mut self) -> Option<Literal> {
        Some(Literal {
            literal_type: LiteralType::Type(String::from("Program")),
            value: Box::new(LiteralValue::NestedValueList(self.statement_list(None))),
        })
    }

    /// Literal
    ///   ; NumericLiteral
    ///   | StringLiteral
    ///   ;
    fn literal(&mut self) -> Option<Literal> {
        return match self.lookahead.clone()?.literal_type {
            LiteralType::Type(ref string) => match string.as_str() {
                "NUMBER" => Some(self.numeric_literal()),
                "STRING" => Some(self.string_literal()),
                _ => panic!("Literal: unexpected literal production."),
            },
        };
    }

    /// Statement List
    ///   : Statement
    ///   | StatementList Statement -> Statement Statement Statement Statement
    ///   ;
    fn statement_list(
        &mut self,
        stop_lookahead: Option<Box<LiteralValue>>,
    ) -> Vec<Option<Literal>> {
        let mut statement_list = vec![self.statement()];
        while let Some(ref lookahead) = self.lookahead {
            if let Some(ref stop_lookahead) = stop_lookahead {
                if lookahead.value == *stop_lookahead {
                    break;
                }
            }
            statement_list.push(self.statement());
        }

        statement_list
    }

    /// Statement
    ///   : ExpressionStatement
    ///   | BlockStatement
    ///   | EmptyStatement
    ///   ;
    fn statement(&mut self) -> Option<Literal> {
        if self.lookahead.clone()?.literal_type == LiteralType::Type(String::from("{")) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    /// BlockStatement
    ///  : '{' OptStatementList '}'
    ///  ;
    fn block_statement(&mut self) -> Option<Literal> {
        self.eat(LiteralType::Type(String::from("{")))
            .expect("Should have been able to eat '{'!");

        let mut body = vec![];
        if self.lookahead.clone()?.literal_type != LiteralType::Type(String::from("}")) {
            body = self.statement_list(Some(Box::new(LiteralValue::Value(String::from("}")))));
        }

        self.eat(LiteralType::Type(String::from("}")))
            .expect("Should have been able to eat '}'!");

        Some(Literal {
            literal_type: LiteralType::Type(String::from("BlockStatement")),
            value: Box::new(LiteralValue::NestedValueList(body)),
        })
    }

    /// ExpressionStatement
    ///   : Expression ';'
    ///   ;
    fn expression_statement(&mut self) -> Option<Literal> {
        let expression = self.expression();
        self.eat(LiteralType::Type(String::from(";")))
            .expect("Should have been able to eat ';'!");

        Some(Literal {
            literal_type: LiteralType::Type(String::from("ExpressionStatement")),
            value: Box::new(LiteralValue::NestedValue(expression)),
        })
    }

    /// Expression
    ///   ; Literal
    ///   ;
    fn expression(&mut self) -> Option<Literal> {
        self.additive_expression()
    }

    /// Additive Expression
    ///   : MultiplicativeExpression
    ///   | AdditiveExpression ADDITIVE_OPERATOR MultiplicativeExpression
    ///   ;
    fn additive_expression(&mut self) -> Option<Literal> {
        self.binary_expression(String::from("ADDITIVE_OPERATOR"))
    }

    /// Multiplicative Expression
    ///   : MultiplicativeExpression
    ///   | MultiplicativeExpression MULTIPLICATIVE_OPERATOR PrimaryExpression
    ///   ;
    fn multiplicative_expression(&mut self) -> Option<Literal> {
        self.binary_expression(String::from("MULTIPLICATIVE_OPERATOR"))
    }

    /// Generic binary expression.
    fn binary_expression(&mut self, operator_token: String) -> Option<Literal> {
        let mut left = match operator_token.as_str() {
            "ADDITIVE_OPERATOR" => self.multiplicative_expression(),
            "MULTIPLICATIVE_OPERATOR" => self.primary_expression(),
            _ => None
        };

        while self.lookahead.clone()?.literal_type
            == LiteralType::Type(operator_token.clone())
        {
            let operator = self
                .eat(LiteralType::Type(operator_token.clone()))
                .expect(format!("Should have been able to eat '{}'!", operator_token).as_str());
            let right = match operator_token.as_str() {
                "ADDITIVE_OPERATOR" => self.multiplicative_expression(),
                "MULTIPLICATIVE_OPERATOR" => self.primary_expression(),
                _ => None
            };

            left = Some(Literal {
                literal_type: LiteralType::Type(String::from("BinaryExpression")),
                value: Box::new(LiteralValue::NestedValueList(vec![
                    Some(Literal {
                        literal_type: LiteralType::Type(String::from("Left")),
                        value: left?.value,
                    }),
                    Some(Literal {
                        literal_type: LiteralType::Type(String::from("Operator")),
                        value: operator.value,
                    }),
                    Some(Literal {
                        literal_type: LiteralType::Type(String::from("Right")),
                        value: right?.value,
                    }),
                ])),
            });
        }

        left
    }

    /// Primary Expression
    ///   : Literal
    ///   | ParenthesisedExpression
    ///   ;
    fn primary_expression(&mut self) -> Option<Literal> {
        match self.lookahead.clone()?.literal_type {
            LiteralType::Type(ref string) => match string.as_str() {
                "(" => self.parenthesised_expression(),
                _ => self.literal()
            }
        }
    }

    /// Parenthesised Expression
    ///   : '(' Expression ')'
    ///   ;
    fn parenthesised_expression(&mut self) -> Option<Literal> {
        self.eat(LiteralType::Type(String::from("(")))
            .expect("Should have been able to eat '('!");
        let expression = self.expression();
        self.eat(LiteralType::Type(String::from(")")))
            .expect("Should have been able to eat ')'!");
        expression
    }

    /// Numeric Literal
    ///   : NUMBER
    ///   ;
    fn numeric_literal(&mut self) -> Literal {
        match self.eat(LiteralType::Type(String::from("NUMBER"))) {
            Ok(token) => Literal {
                literal_type: LiteralType::Type(String::from("NumericLiteral")),
                value: token.value,
            },

            Err(err) => panic!("{}", err),
        }
    }

    /// String Literal
    ///   : STRING
    ///   ;
    fn string_literal(&mut self) -> Literal {
        match self.eat(LiteralType::Type(String::from("STRING"))) {
            Ok(token) => Literal {
                literal_type: LiteralType::Type(String::from("StringLiteral")),
                value: token.value,
            },

            Err(err) => panic!("{}", err),
        }
    }

    fn eat(&mut self, token_type: LiteralType) -> Result<Literal, String> {
        if let Some(token) = self.lookahead.clone() {
            if token.literal_type != token_type {
                return Err(format!(
                    "Unexpected token: {:?}, expected: {:?}",
                    token.value, token_type
                ));
            }
            // Advance to next token
            self.lookahead = self.tokenizer.get_next_token();

            return Ok(token);
        }

        Err(format!(
            "Unexpected end of input, expected: {:?}",
            token_type
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::*;

    #[test]
    fn test_statement_lists() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            // Program
            /*
                Multiline comments...
            */
            "hello";
            42;
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![
                    Some(Literal {
                        literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                        value: Box::new(LiteralValue::NestedValue(Some(Literal {
                            literal_type: LiteralType::Type(String::from("StringLiteral")),
                            value: Box::new(LiteralValue::Value(String::from("\"hello\"")))
                        })))
                    }),
                    Some(Literal {
                        literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                        value: Box::new(LiteralValue::NestedValue(Some(Literal {
                            literal_type: LiteralType::Type(String::from("NumericLiteral")),
                            value: Box::new(LiteralValue::Value(String::from("42")))
                        })))
                    })
                ]))
            })
        );
        dbg!(ast);
    }

    #[test]
    fn test_string_literals() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            "hello";
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                    literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                    value: Box::new(LiteralValue::NestedValue(Some(Literal {
                        literal_type: LiteralType::Type(String::from("StringLiteral")),
                        value: Box::new(LiteralValue::Value(String::from("\"hello\"")))
                    })))
                })]))
            })
        );
        dbg!(ast);
    }

    #[test]
    fn test_numeric_literals() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            42;
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                    literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                    value: Box::new(LiteralValue::NestedValue(Some(Literal {
                        literal_type: LiteralType::Type(String::from("NumericLiteral")),
                        value: Box::new(LiteralValue::Value(String::from("42")))
                    })))
                })]))
            })
        );
        dbg!(ast);
    }

    #[test]
    fn test_blocks() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            {
              42;
              "hello";
            }
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                    literal_type: LiteralType::Type(String::from("BlockStatement")),
                    value: Box::new(LiteralValue::NestedValueList(vec![
                        Some(Literal {
                            literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                            value: Box::new(LiteralValue::NestedValue(Some(Literal {
                                literal_type: LiteralType::Type(String::from("NumericLiteral")),
                                value: Box::new(LiteralValue::Value(String::from("42")))
                            })))
                        }),
                        Some(Literal {
                            literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                            value: Box::new(LiteralValue::NestedValue(Some(Literal {
                                literal_type: LiteralType::Type(String::from("StringLiteral")),
                                value: Box::new(LiteralValue::Value(String::from("\"hello\"")))
                            })))
                        })
                    ]))
                })]))
            })
        );
        dbg!(ast);
    }

    #[test]
    fn test_empty_block() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            {
              // ...
            }
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                    literal_type: LiteralType::Type(String::from("BlockStatement")),
                    value: Box::new(LiteralValue::NestedValueList(vec![]))
                })]))
            })
        );
        dbg!(ast);
    }

    #[test]
    fn test_nested_blocks() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            {
              42;
              {
                "hello";
              }
            }
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                    literal_type: LiteralType::Type(String::from("BlockStatement")),
                    value: Box::new(LiteralValue::NestedValueList(vec![
                        Some(Literal {
                            literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                            value: Box::new(LiteralValue::NestedValue(Some(Literal {
                                literal_type: LiteralType::Type(String::from("NumericLiteral")),
                                value: Box::new(LiteralValue::Value(String::from("42")))
                            })))
                        }),
                        Some(Literal {
                            literal_type: LiteralType::Type(String::from("BlockStatement")),
                            value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                                literal_type: LiteralType::Type(String::from(
                                    "ExpressionStatement"
                                )),
                                value: Box::new(LiteralValue::NestedValue(Some(Literal {
                                    literal_type: LiteralType::Type(String::from("StringLiteral")),
                                    value: Box::new(LiteralValue::Value(String::from("\"hello\"")))
                                })))
                            })]))
                        })
                    ]))
                })]))
            })
        );
        dbg!(ast);
    }

    #[test]
    fn test_math() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            2 + 2 * 2;
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                    literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                    value: Box::new(LiteralValue::NestedValue(Some(Literal {
                        literal_type: LiteralType::Type(String::from("BinaryExpression")),
                        value: Box::new(LiteralValue::NestedValueList(vec![
                            Some(Literal {
                                literal_type: LiteralType::Type(String::from("Left")),
                                value: Box::new(LiteralValue::Value(String::from("2")))
                            }),
                            Some(Literal {
                                literal_type: LiteralType::Type(String::from("Operator")),
                                value: Box::new(LiteralValue::Value(String::from("+")))
                            }),
                            Some(Literal {
                                literal_type: LiteralType::Type(String::from("Right")),
                                value: Box::new(LiteralValue::NestedValueList(vec![
                                    Some(Literal {
                                        literal_type: LiteralType::Type(String::from("Left")),
                                        value: Box::new(LiteralValue::Value(String::from("2")))
                                    }),
                                    Some(Literal {
                                        literal_type: LiteralType::Type(String::from("Operator")),
                                        value: Box::new(LiteralValue::Value(String::from("*")))
                                    }),
                                    Some(Literal {
                                        literal_type: LiteralType::Type(String::from("Right")),
                                        value: Box::new(LiteralValue::Value(String::from("2")))
                                    }),
                                ]))
                            })
                        ]))
                    })))
                })]))
            })
        );
        dbg!(ast);
    }

    #[test]
    fn test_op_precedence() {
        let mut parser = Parser::new();

        let program: String = String::from(
            r#"
            (2 + 2) * 2;
            "#,
        );

        let ast = parser.parse(program);

        assert_eq!(
            ast,
            Some(Literal {
                literal_type: LiteralType::Type(String::from("Program")),
                value: Box::new(LiteralValue::NestedValueList(vec![Some(Literal {
                    literal_type: LiteralType::Type(String::from("ExpressionStatement")),
                    value: Box::new(LiteralValue::NestedValue(Some(Literal {
                        literal_type: LiteralType::Type(String::from("BinaryExpression")),
                        value: Box::new(LiteralValue::NestedValueList(vec![
                            Some(Literal {
                                literal_type: LiteralType::Type(String::from("Left")),
                                value: Box::new(LiteralValue::NestedValueList(vec![
                                    Some(Literal {
                                        literal_type: LiteralType::Type(String::from("Left")),
                                        value: Box::new(LiteralValue::Value(String::from("2")))
                                    }),
                                    Some(Literal {
                                        literal_type: LiteralType::Type(String::from("Operator")),
                                        value: Box::new(LiteralValue::Value(String::from("+")))
                                    }),
                                    Some(Literal {
                                        literal_type: LiteralType::Type(String::from("Right")),
                                        value: Box::new(LiteralValue::Value(String::from("2")))
                                    }),
                                ]))
                            }),
                            Some(Literal {
                                literal_type: LiteralType::Type(String::from("Operator")),
                                value: Box::new(LiteralValue::Value(String::from("*")))
                            }),
                            Some(Literal {
                                literal_type: LiteralType::Type(String::from("Right")),
                                value: Box::new(LiteralValue::Value(String::from("2")))
                            }),
                        ]))
                    })))
                })]))
            })
        );
        dbg!(ast);
    }
}

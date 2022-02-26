# NeaLang Syntax Specification

This document outlines the syntax and syntax only for NeaLang. It uses a slightly modified version of EBNF.
The syntax used is as follows:

- `"x"` - Refers to a token with the exact string `"x"`
- `[ x ]` - Refers to zero or one `x`
- `{ x }` - Refers to zero or more `x`
- `x, y` - Refers to `x` followed by a `y`, ignoring whitespace
- `( x )` - Closes the scope of `x` to within the parenthesis
- `x | y` - Refers to `x` or `y`, with `|` running up to the first `[]`, `{}` or `()`
- `{ x }y` - Refers to zero or more `x` delimited by `y`, equivalent to `[ x { y, x } ]`
- Lowercase (e.g. `ident`) names refer to token types
- Capitalised names (e.g. `TypeExpr`) refer to patterns

Note that the structure of parsing within this file is not necessarily identical to how it is parsed in Rust, it is here only to show what the result should be.

## TranslationUnit
The translation unit is the root of parsing - it represents the result of parsing a single source file.
```js
TranslationUnit ::= { TopLevelNode } ;
TopLevelNode ::= ImportStmt | StructDecl | FunctionDecl ;
```

## TypeExpr
```js
TypeExpr ::= ident, { ".", ident }, [ "[", Expr, "]" ];
```

## ImportStmt
```js
ImportStmt ::= "import", ident, { ".", ident } ;
```

## StructDecl
```js
StructDecl ::= "struct", ident, "{", { StructFieldDecl }",", "}" ;
StructFieldDecl ::= name, ":", TypeExpr ;
```

## FunctionDecl
```js
FunctionDecl ::= "func", [ FunctionAnnotations ], FunctionIdentifier, "(", FunctionParams, ")", [ ":", FunctionReturnTypes ] FunctionCode ;

FunctionAnnotations ::= "[", { FunctionAnnotation }",", "]" ;
FunctionAnnotation ::= ident, "=", Expr ;

FunctionIdentifier ::= { ident, "." }, ident ;

FunctionParams ::= { FirstFunctionParam }"," ;
FirstFunctionParam ::= "self" | FunctionParam ;
FunctionParam ::= ident, ":", TypeExpr ;

FunctionReturnTypes ::= "(", { TypeExpr }",", ")" | TypeExpr;

FunctionCode ::= "extern" | "{", { Code }, "}" ;
```

## Code
```js
Code := ReturnStmt | VarDeclaration | ExprStmt | Assignment | IfStmt | ForStmt ;
CodeBlock := "{", { Code }, "}" | Code ;
```

## ReturnStmt
```js
ReturnStmt ::= "return", [ Expr ], ";" ;
```

## VarDeclaration
```js
VarDeclaration ::= "var", ident, [ ":", TypeExpr ], ["=", Expr ], ";" ;
```

## ExprStmt
```js
ExprStmt ::= Expr, ";" ;
```

## Assignment
```js
Assignment ::= Expr, "=", Expr, ";" ;
```

## IfStmt
```js
IfStmt := "if", Expr, CodeBlock, [ "else", CodeBlock ] ;
```

## ForStmt
```js
ForStmt := "for", ForStmtInitCondInc, CodeBlock ;
ForStmtInitCondInc := Expr | [ Code ] ";" [ Expr ] ";" [ Code ] ;
```

## Expr
```js
Expr ::= BoolExpr ;

BoolExpr ::= CmpExpr | CmpExpr, ( "&&" | "||" ), CmpExpr ;
CmpExpr ::= AddSubExpr | AddSubExpr, ( "==" | "!=" | ">" | ">=" | "<" | "<=" ), AddSubExpr ;
AddSubExpr ::= MulDivExpr | MulDivExpr, ( "+" | "-" ), MulDivExpr ;
MulDivExpr ::= PrimaryExpr | PrimaryExpr, ( "*" | "/" ), PrimaryExpr ;

PrimaryExpr ::= PrimaryLeftExpr | CallExpr | IndexExpr | MemberAccessExpr | AsExpr ;
CallExpr ::= PrimaryExpr, "(", { Expr }",", ")" ;
IndexExpr ::= PrimaryExpr, "[", Expr, "]" ;
MemberAccessExpr ::= PrimaryExpr, ".", ident ;
AsExpr ::= PrimaryExpr, "as", TypeExpr ;

PrimaryLeftExpr ::= ClosedExpr | NumberLitExpr | StringLitExpr | IdentExpr | NewExpr | SliceLitExpr | BoolLitExpr ;
ClosedExpr ::= "(", Expr, ")" ;
NumberLitExpr ::= number ;
StringLitExpr ::= string ;
IdentExpr ::= ident | "self" ;
NewExpr ::= "new", TypeExpr ;
SliceLitExpr ::= "[", { Expr }",", "]" ;
BoolLitExpr ::= "true" | "false" ;
```

The layers used in `Expr` exist to implement operator precedence. For example it parses `6 / 3 + 1` as equivalent to `(6 / 3) + 1` and not as `6 / (3 + 1)` as it would otherwise be.

use syntax::MatchResult;

use crate::lexer::*;
use crate::ast::*;

#[test]
fn functions() {
    let string = "
func [attribute=something, other_attribute=1+2, final] function(param: String, param2: i32, param3: x.y.z): (i32, x.y) {
    var x = 1;
}

func something_extern() extern

func a.b.c.something() {}
func a.b.c.something_else(self) {}
    ";

    let mut tokenstream = TokenStream::new(string, Box::new(Matcher));
    tokenstream.step();

    let result = match TranslationUnit::parse(&mut tokenstream) {
        MatchResult::Ok(unit) => unit,
        _ => panic!("Did not parse")
    };

    assert_eq!(result.nodes.len(), 4);
    let mut func = match &result.nodes[0] {
          TopLevelNode::Function(func) => func,
          _ => panic!()
    };

    assert_eq!(func.annotations.len(), 3);
    
    assert_eq!(func.annotations[0].name, "attribute");
    assert_eq!(func.annotations[1].name, "other_attribute");
    assert_eq!(func.annotations[2].name, "final");
    assert!(matches!(&func.annotations[0].value, Some(Expr::Name(e)) if e.name == "something"));
    match &func.annotations[1].value {
        Some(Expr::BinaryExpr(BinaryExpr { op, left, right, .. })) => {
            assert!(matches!(op, BinaryOp::Add));
            assert!(matches!(left.as_ref(), Expr::NumberLit(expr) if expr.number == "1"));
            assert!(matches!(right.as_ref(), Expr::NumberLit(expr) if expr.number == "2"));
        },
        _ => panic!()
    };
    assert!(func.annotations[2].value.is_none());

    assert_eq!(func.name, "function");
    assert_eq!(func.params.len(), 3);
    assert!(func.is_static);

    assert!(func.params[0].name == "param");
    assert!(func.params[0].param_type.path == &["String"]);
    assert!(func.params[1].name == "param2");
    assert!(func.params[1].param_type.path == &["i32"]);
    assert!(func.params[2].name == "param3");
    assert!(func.params[2].param_type.path == &["x", "y", "z"]);

    assert_eq!(func.return_types.len(), 2);
    assert!(func.return_types[0].path == &["i32"]);
    assert!(func.return_types[1].path == &["x", "y"]);

    assert!(func.code.is_some());
    assert_eq!(func.code.as_ref().unwrap().len(), 1);
    assert!(matches!(&func.code.as_ref().unwrap()[0], Code::VarDeclaration(VarDeclaration { name, .. }) if name == "x"));

    func = match &result.nodes[1] {
        TopLevelNode::Function(func) => func,
        _ => panic!()
    };

    assert_eq!(func.name, "something_extern");
    assert_eq!(func.params.len(), 0);
    assert_eq!(func.return_types.len(), 0);
    assert!(func.code.is_none());
    assert!(func.is_static);

    func = match &result.nodes[2] {
        TopLevelNode::Function(func) => func,
        _ => panic!()
    };

    assert_eq!(func.name, "something");
    assert_eq!(func.path, &["a", "b", "c"]);
    assert_eq!(func.params.len(), 0);
    assert_eq!(func.return_types.len(), 0);
    assert!(func.code.is_some());
    assert!(func.is_static);

    func = match &result.nodes[3] {
        TopLevelNode::Function(func) => func,
        _ => panic!()
    };

    assert_eq!(func.name, "something_else");
    assert_eq!(func.path, &["a", "b", "c"]);
    assert_eq!(func.params.len(), 0);
    assert_eq!(func.return_types.len(), 0);
    assert!(!func.is_static);
}
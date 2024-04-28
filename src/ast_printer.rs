use crate::{
    expr::{Acceptor as ExprAcceptor, Expr, Visitor as ExprVisitor},
    stmt::{Acceptor as StmtAcceptor, Stmt, Visitor as StmtVisitor},
    token::{Literal, Token},
};

#[derive(Debug, Clone, Copy)]
pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> String {
        expr.accept(self)
    }

    pub fn print_statements(&mut self, statements: Vec<Stmt>) -> String {
        let mut string = String::new();
        for statement in statements {
            string.push_str(&statement.accept(self));
            string.push_str("\n");
        }
        string
    }

    fn parenthesize(&mut self, name: String, exprs: Vec<Expr>) -> String {
        let mut string = String::new();
        string.push_str("(");
        string.push_str(&name);
        for expr in exprs {
            string.push_str(" ");
            string.push_str(&expr.accept(self))
        }
        string.push_str(")");
        string
    }
}

impl ExprVisitor<String> for AstPrinter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.parenthesize(operator.lexeme.clone(), vec![left.clone(), right.clone()])
    }
    fn visit_grouping(&mut self, expr: &Expr) -> String {
        self.parenthesize(String::from("group"), vec![expr.clone()])
    }
    fn visit_literal(&mut self, expr: &Literal) -> String {
        match expr {
            Literal::None => String::from("nil"),
            Literal::Isize(u) => u.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::String(s) => String::from(s),
            Literal::Bool(b) => b.to_string(),
        }
    }
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> String {
        self.parenthesize(operator.lexeme.clone(), vec![right.clone()])
    }

    fn visit_variable(&mut self, name: &Token) -> String {
        todo!()
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr) -> String {
        todo!()
    }

    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        todo!()
    }

    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) -> String {
        todo!()
    }

    fn visit_get(&mut self, object: &Expr, name: &Token) -> String {
        todo!()
    }

    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> String {
        todo!()
    }

    fn visit_this(&mut self, keyword: &Token) -> String {
        todo!()
    }

    fn visit_super(&mut self, keyword: &Token, method: &Token) -> String {
        todo!()
    }
}

impl StmtVisitor<String> for AstPrinter {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> String {
        expression.accept(self)
    }
    fn visit_print_stmt(&mut self, expression: &Expr) -> String {
        let mut string = String::new();
        string.push_str("(print ");
        string.push_str(&expression.accept(self));
        string.push_str(")");
        string
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> String {
        todo!()
    }

    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> String {
        todo!()
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> String {
        todo!()
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> String {
        todo!()
    }

    fn visit_function_stmt(&mut self, name: &Token, params: &[Token], body: &[Stmt]) -> String {
        todo!()
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Expr) -> String {
        todo!()
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        super_class: &Option<Expr>,
        methods: &[Stmt],
    ) -> String {
        todo!()
    }
}

use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    // Struct
    // Protocol
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    // list of parameters
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub tail_expression: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Unary {
        operator: UnaryOperator,
        expression: Box<Self>,
    },
    Binary {
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
    },
    Return(Option<Box<Self>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Int(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Named(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negation, // -
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

struct Node {
    label: String,
    children: Vec<Self>,
}

impl Node {
    fn new(label: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            label: label.into(),
            children,
        }
    }

    fn render(&self, out: &mut String) {
        out.push_str(&self.label);
        out.push('\n');
        self.render_children(out, "");
    }

    fn render_children(&self, out: &mut String, prefix: &str) {
        let last = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().enumerate() {
            let is_last = i == last;
            let connector = if is_last { "└─ " } else { "├─ " };
            let _ = writeln!(out, "{prefix}{connector}{}", child.label);

            let next_prefix = format!("{prefix}{}", if is_last { "   " } else { "│  " });
            child.render_children(out, &next_prefix);
        }
    }
}

impl Program {
    pub fn dump_tree(&self) -> String {
        let mut out = String::new();
        self.to_node().render(&mut out);
        out
    }

    fn to_node(&self) -> Node {
        Node::new("Program", self.items.iter().map(Item::to_node).collect())
    }
}

impl Item {
    fn to_node(&self) -> Node {
        match self {
            Self::Function(function) => function.to_node(),
        }
    }
}

impl Function {
    fn to_node(&self) -> Node {
        let mut children = Vec::new();
        if let Some(ty) = &self.return_type {
            children.push(Node::new(format!("ReturnType {ty:?}"), vec![]));
        }
        children.push(self.body.to_node());
        Node::new(format!("Function {:?}", self.name), children)
    }
}

impl Block {
    fn to_node(&self) -> Node {
        let mut children: Vec<Node> = self.statements.iter().map(Statement::to_node).collect();
        if let Some(tail) = &self.tail_expression {
            children.push(Node::new("Tail", vec![tail.to_node()]));
        }
        Node::new("Block", children)
    }
}

impl Statement {
    fn to_node(&self) -> Node {
        match self {
            Self::Expression(e) => Node::new("Expression", vec![e.to_node()]),
        }
    }
}

impl Expression {
    fn to_node(&self) -> Node {
        match self {
            Self::Literal(Literal::Int(n)) => Node::new(format!("Int {n}"), vec![]),
            Self::Unary {
                operator,
                expression,
            } => Node::new(format!("Unary {operator:?}"), vec![expression.to_node()]),
            Self::Binary {
                left,
                operator,
                right,
            } => Node::new(
                format!("Binary {operator:?}"),
                vec![left.to_node(), right.to_node()],
            ),
            Self::Return(expr) => Node::new("Return", expr.iter().map(|e| e.to_node()).collect()),
        }
    }
}

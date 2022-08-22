pub struct Program<'a> {
	pub functions: Vec<Function<'a>>,
}

impl <'a> Program<'a> {
	pub fn new() -> Self {
		Program {
			functions: Vec::new(),
		}
	}
	pub fn get_main_function(&self) -> Option<&Function<'a>> {
		for function in &self.functions {
			if function.name == "main" {
				return Some(function)
			}
		}
		None
	}
}

pub struct Function<'a> {
	pub name: &'a str,
	pub arguments: Vec<&'a str>,
	pub statements: Vec<Statement<'a>>,
}

pub enum Statement<'a> {
	If(If<'a>),
	While(While<'a>),
	Return(Expression<'a>),
	Expression(Expression<'a>),
}

pub struct If<'a> {
	pub condition: Box<Expression<'a>>,
	pub statements: Vec<Statement<'a>>,
}

pub struct While<'a> {
	pub condition: Box<Expression<'a>>,
	pub statements: Vec<Statement<'a>>,
}

pub enum Expression<'a> {
	Number(&'a str),
	Name(&'a str),
	ArithmeticExpression(ArithmeticExpression<'a>),
	RelationalExpression(RelationalExpression<'a>),
	Assign(Box<Expression<'a>>, Box<Expression<'a>>),
}

pub struct ArithmeticExpression<'a> {
	pub operation: ArithmeticOperation,
	pub left: Box<Expression<'a>>,
	pub right: Box<Expression<'a>>,
}

pub enum ArithmeticOperation {
	Add,
	Subtract,
	Multiply,
	Divide,
	Remainder,
}

pub struct RelationalExpression<'a> {
	pub operation: RelationalOperation,
	pub left: Box<Expression<'a>>,
	pub right: Box<Expression<'a>>,
}

pub enum RelationalOperation {
	Equal,
	NotEqual,
	LessThan,
	LessThanOrEqual,
	GreaterThan,
	GreaterThanOrEqual,
}

impl <'a> Expression<'a> {
	pub fn add<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Add,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn subtract<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Subtract,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn multiply<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Multiply,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn divide<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Divide,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn remainder<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Remainder,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn equal<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::Equal,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn not_equal<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::NotEqual,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn less_than<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::LessThan,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn less_than_or_equal<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::LessThanOrEqual,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn greater_than<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::GreaterThan,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn greater_than_or_equal<'b>(left: Expression<'b>, right: Expression<'b>) -> Expression<'b> {
		Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::GreaterThanOrEqual,
			left: Box::new(left),
			right: Box::new(right),
		})
	}
	pub fn assign<'b>(name: Expression<'b>, expression: Expression<'b>) -> Expression<'b> {
		Expression::Assign(Box::new(name), Box::new(expression))
	}
}

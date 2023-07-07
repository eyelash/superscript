use crate::error::Location;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type<'a> {
	Number,
	Boolean,
	Void,
	Class(&'a str),
}

pub struct Program<'a> {
	pub functions: Vec<Function<'a>>,
	pub classes: Vec<Class<'a>>,
	pub locations: std::collections::HashMap<* const Expression<'a>, Location>,
}

impl <'a> Program<'a> {
	pub fn new() -> Self {
		Program {
			functions: Vec::new(),
			classes: Vec::new(),
			locations: std::collections::HashMap::new(),
		}
	}
	pub fn get_function(&self, name: &str) -> Option<&Function<'a>> {
		for function in &self.functions {
			if function.name == name {
				return Some(function);
			}
		}
		None
	}
	pub fn get_main_function(&self) -> Option<&Function<'a>> {
		self.get_function("main")
	}
	pub fn get_class(&self, name: &str) -> Option<&Class<'a>> {
		for class in &self.classes {
			if class.name == name {
				return Some(class);
			}
		}
		None
	}
}

pub struct Function<'a> {
	pub name: &'a str,
	pub arguments: Vec<(&'a str, Type<'a>)>,
	pub return_type: Type<'a>,
	pub statements: Vec<Statement<'a>>,
}

pub struct Class<'a> {
	pub name: &'a str,
	pub fields: Vec<(&'a str, Type<'a>)>,
	pub methods: Vec<Function<'a>>,
}

impl <'a> Class<'a> {
	pub fn get_method(&self, name: &str) -> Option<&Function<'a>> {
		for method in &self.methods {
			if method.name == name {
				return Some(method);
			}
		}
		None
	}
	pub fn get_constructor(&self) -> Option<&Function<'a>> {
		self.get_method("constructor")
	}
	pub fn get_field(&self, name: &str) -> Option<Type<'a>> {
		for (field_name, field_type) in &self.fields {
			if field_name == &name {
				return Some(field_type.clone());
			}
		}
		None
	}
}

pub enum Statement<'a> {
	VariableDeclaration {
		name: &'a str,
		expression: Box<Expression<'a>>,
	},
	If(If<'a>),
	While(While<'a>),
	Return(Box<Expression<'a>>),
	Expression(Box<Expression<'a>>),
	Block(Vec<Statement<'a>>),
}

pub struct If<'a> {
	pub condition: Box<Expression<'a>>,
	pub statement: Box<Statement<'a>>,
	pub else_statement: Option<Box<Statement<'a>>>,
}

pub struct While<'a> {
	pub condition: Box<Expression<'a>>,
	pub statement: Box<Statement<'a>>,
}

pub enum Expression<'a> {
	Number(&'a str),
	Name(&'a str),
	ArithmeticExpression(ArithmeticExpression<'a>),
	RelationalExpression(RelationalExpression<'a>),
	LogicalExpression(LogicalExpression<'a>),
	Not(Box<Expression<'a>>),
	Assign {
		name: Box<Expression<'a>>,
		expression: Box<Expression<'a>>,
	},
	Call {
		function: Box<Expression<'a>>,
		arguments: Vec<Box<Expression<'a>>>,
	},
	ClassInstantiation {
		class: &'a str,
		arguments: Vec<Box<Expression<'a>>>,
	},
	PropertyAccess {
		object: Box<Expression<'a>>,
		property: &'a str,
	},
	MethodCall {
		object: Box<Expression<'a>>,
		method: &'a str,
		arguments: Vec<Box<Expression<'a>>>,
	},
	This,
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

pub struct LogicalExpression<'a> {
	pub operation: LogicalOperation,
	pub left: Box<Expression<'a>>,
	pub right: Box<Expression<'a>>,
}

pub enum LogicalOperation {
	And,
	Or,
}

impl <'a> Expression<'a> {
	pub fn add<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Add,
			left,
			right,
		}))
	}
	pub fn subtract<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Subtract,
			left,
			right,
		}))
	}
	pub fn multiply<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Multiply,
			left,
			right,
		}))
	}
	pub fn divide<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Divide,
			left,
			right,
		}))
	}
	pub fn remainder<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::ArithmeticExpression(ArithmeticExpression {
			operation: ArithmeticOperation::Remainder,
			left,
			right,
		}))
	}
	pub fn equal<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::Equal,
			left,
			right,
		}))
	}
	pub fn not_equal<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::NotEqual,
			left,
			right,
		}))
	}
	pub fn less_than<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::LessThan,
			left,
			right,
		}))
	}
	pub fn less_than_or_equal<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::LessThanOrEqual,
			left,
			right,
		}))
	}
	pub fn greater_than<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::GreaterThan,
			left,
			right,
		}))
	}
	pub fn greater_than_or_equal<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::RelationalExpression(RelationalExpression {
			operation: RelationalOperation::GreaterThanOrEqual,
			left,
			right,
		}))
	}
	pub fn and<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::LogicalExpression(LogicalExpression {
			operation: LogicalOperation::And,
			left,
			right,
		}))
	}
	pub fn or<'b>(left: Box<Expression<'b>>, right: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::LogicalExpression(LogicalExpression {
			operation: LogicalOperation::Or,
			left,
			right,
		}))
	}
	pub fn not<'b>(expression: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::Not(expression))
	}
	pub fn assign<'b>(name: Box<Expression<'b>>, expression: Box<Expression<'b>>) -> Box<Expression<'b>> {
		Box::new(Expression::Assign {
			name,
			expression,
		})
	}
}

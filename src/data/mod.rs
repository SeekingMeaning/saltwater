pub mod types;

use std::collections::{HashMap, VecDeque};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Display, Formatter, Write};
use std::hash::Hash;

use cranelift::codegen::ir::condcodes::{FloatCC, IntCC};

use self::types::{TypeIndex, Types};
use crate::backend::SIZE_T;

pub type SemanticResult<T> = Result<T, Locatable<String>>;

pub mod prelude {
    pub use super::types::{TypeIndex, Types};
    pub use super::{
        Declaration, Expr, ExprType, Locatable, Location, SemanticResult, Stmt, StmtType, Symbol,
        Token, Type,
    };
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Keyword {
    // statements
    If,
    Else,
    Do,
    While,
    For,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Return,
    Goto,

    // types
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Void,
    Signed,
    Unsigned,
    Typedef,
    Union,
    Struct,
    Enum,
    // weird types
    Bool,
    Complex,
    Imaginary,
    VaList,

    // qualifiers
    Const,
    Volatile,
    Restrict,
    // weird qualifiers
    Atomic,
    ThreadLocal,
    // function qualifiers
    Inline,
    NoReturn,

    // storage classes
    Auto,
    Register,
    Static,
    Extern,

    // intrinsics
    Sizeof,
    Generic,
    StaticAssert,
    Alignas,
    Alignof,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    PlusPlus,
    MinusMinus,

    PlusEqual,
    MinusEqual,
    StarEqual,
    DivideEqual,
    ModEqual,
    LeftEqual,  // <<=
    RightEqual, // >>=
    AndEqual,
    OrEqual,
    XorEqual, // ^=

    EqualEqual,
    NotEqual,
    LessEqual,
    GreaterEqual,

    Plus,
    Minus,
    Star,
    Divide,
    Mod,
    Xor,
    Equal,
    Less,
    Greater,
    Ampersand,
    LogicalAnd,
    BitwiseOr,
    LogicalOr,
    BinaryNot,  // ~
    LogicalNot, // !
    ShiftRight,
    ShiftLeft,

    LeftBrace, // {
    RightBrace,
    LeftBracket, // [
    RightBracket,
    LeftParen,
    RightParen,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Question,

    // literals
    Int(i64),
    UnsignedInt(u64),
    Float(f64),
    Str(String),
    Char(u8),
    Id(String),

    Keyword(Keyword),

    // Misc
    Ellipsis,
    StructDeref, // ->
}

pub type Stmt = Locatable<StmtType>;

#[derive(Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StmtType {
    Compound(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Do(Box<Stmt>, Expr),
    While(Expr, Option<Box<Stmt>>),
    // for(int i = 1, j = 2; i < 4; ++i) body
    For(
        VecDeque<Locatable<Declaration>>,
        Box<Expr>,
        Box<Expr>,
        Box<Expr>,
    ),
    Switch(Expr, Box<Stmt>),
    Label(String, Option<Box<Stmt>>),
    Case(Expr),
    Default,
    Expr(Expr),
    Goto(String),
    Continue,
    Break,
    Return(Option<Expr>),
    Decl(VecDeque<Locatable<Declaration>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub symbol: Symbol,
    pub init: Option<Initializer>,
}

#[derive(Clone, PartialEq)]
pub enum Initializer {
    Scalar(Expr),                      // int i = 5;
    InitializerList(Vec<Initializer>), // int a[] = { 1, 2, 3 };
    FunctionBody(Vec<Stmt>),           // int f() { return 0; }
}

/// Holds the metadata for an expression.
///
/// This should be the datatype you use in APIs, etc.
/// because it is more useful than the raw ExprType.
#[derive(Clone, PartialEq)]
pub struct Expr {
    /// expr: holds the actual expression
    pub expr: ExprType,

    /// ctype: holds the type of the expression
    pub ctype: TypeIndex,

    /// constexpr: whether a value can be constant-folded at compile-time
    ///
    /// unrelated to the `const` keyword
    /// NOTE: can sometimes be true at the same time as `lval` (e.g. for constant arrays)
    pub constexpr: bool,

    /// lval: whether an expression can be assigned to
    ///
    /// for example, variables, array elements, and pointer dereferences are lvals,
    /// but literals, functions, and addresses cannot
    pub lval: bool,

    /// location: the best approximation of where the expression is
    ///
    /// usually points to the location of the operation symbol, or the literal if no
    /// operations is being performed
    /// implicit operations should point to the child expression
    pub location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprType {
    Id(Symbol),
    Literal(Token),
    FuncCall(Box<Expr>, Vec<Expr>),
    Member(Box<Expr>, Token),
    // pre/post inc/dec-rement
    Increment(Box<Expr>, bool, bool),
    Cast(Box<Expr>),
    Sizeof(TypeIndex),
    Deref(Box<Expr>),
    Negate(Box<Expr>),
    LogicalNot(Box<Expr>),
    BitwiseNot(Box<Expr>),
    LogicalOr(Box<Expr>, Box<Expr>),
    BitwiseOr(Box<Expr>, Box<Expr>),
    LogicalAnd(Box<Expr>, Box<Expr>),
    BitwiseAnd(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    // bool: left or right
    Shift(Box<Expr>, Box<Expr>, bool),
    // Token: make >, <, <=, ... part of the same variant
    Compare(Box<Expr>, Box<Expr>, Token),
    // Token: allow extended assignment
    Assign(Box<Expr>, Box<Expr>, Token),
    // Ternary: if ? then : else
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Comma(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Bool,
    Char(bool), // signed or unsigned
    Short(bool),
    Int(bool),
    Long(bool),
    Float,
    Double,
    Pointer(Box<Type>, Qualifiers),
    Array(Box<Type>, ArrayType),
    Function(FunctionType),
    // name, members
    // no members means a tentative definition (struct s;)
    Union(Option<String>, Vec<Symbol>),
    Struct(Option<String>, Vec<Symbol>),
    // enums should always have members, since tentative definitions are not allowed
    Enum(Option<String>, Vec<(String, i64)>),
    Bitfield(Vec<BitfieldType>),
    VaList,
}

#[derive(Clone, Debug)]
pub enum ArrayType {
    Fixed(SIZE_T),
    Unbounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageClass {
    Static = Keyword::Static as isize,
    Extern = Keyword::Extern as isize,
    Auto = Keyword::Auto as isize,
    Register = Keyword::Register as isize,
    Typedef = Keyword::Typedef as isize,
}

/* structs */
#[derive(Clone, Debug)]
pub struct Symbol {
    pub id: String,
    pub ctype: TypeIndex,
    pub qualifiers: Qualifiers,
    pub storage_class: StorageClass,
    pub init: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Qualifiers {
    pub volatile: bool,
    pub c_const: bool,
}

#[derive(Clone, Debug, Eq)]
// note: old-style declarations are not supported at this time
pub struct FunctionType {
    // why Symbol instead of Type?
    // 1. we need to know qualifiers for the params. if we made that part of Type,
    //    we'd need qualifiers for every step along the way
    //    (consider that int a[][][] parses as 4 nested types).
    // 2. when we do scoping, we need to know the names of formal parameters
    //    (as opposed to concrete arguments).
    //    this is as good a place to store them as any.
    pub return_type: TypeIndex,
    pub params: Vec<Symbol>,
    pub varargs: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitfieldType {
    pub offset: i32,
    pub name: Option<String>,
    pub ctype: TypeIndex,
}

#[derive(Debug)]
pub struct Scope<K: Hash + Eq, V>(Vec<HashMap<K, V>>);

// holds where a piece of code came from
// should almost always be immutable
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Location {
    // if there's a 4 GB input file, we have bigger problems
    pub line: u32,
    pub column: u32,
    pub file: String,
}

#[derive(Clone, Debug)]
pub struct Locatable<T> {
    pub data: T,
    pub location: Location,
}

/* impls */
impl<T: PartialEq> PartialEq for Locatable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T: Eq> Eq for Locatable<T> {}

#[allow(dead_code)]
impl Qualifiers {
    pub const NONE: Qualifiers = Qualifiers {
        c_const: false,
        volatile: false,
    };
    pub const VOLATILE: Qualifiers = Qualifiers {
        c_const: false,
        volatile: true,
    };
    pub const CONST: Qualifiers = Qualifiers {
        c_const: true,
        volatile: false,
    };
    pub const CONST_VOLATILE: Qualifiers = Qualifiers {
        c_const: true,
        volatile: true,
    };
}

impl Token {
    pub fn to_int_compare(&self, signed: bool) -> Result<IntCC, ()> {
        match (self, signed) {
            (Token::Less, true) => Ok(IntCC::SignedLessThan),
            (Token::Less, false) => Ok(IntCC::UnsignedLessThan),
            (Token::LessEqual, true) => Ok(IntCC::SignedLessThanOrEqual),
            (Token::LessEqual, false) => Ok(IntCC::UnsignedLessThanOrEqual),
            (Token::Greater, true) => Ok(IntCC::SignedGreaterThan),
            (Token::Greater, false) => Ok(IntCC::UnsignedGreaterThan),
            (Token::GreaterEqual, true) => Ok(IntCC::SignedGreaterThanOrEqual),
            (Token::GreaterEqual, false) => Ok(IntCC::UnsignedGreaterThanOrEqual),
            (Token::EqualEqual, _) => Ok(IntCC::Equal),
            (Token::NotEqual, _) => Ok(IntCC::NotEqual),
            _ => Err(()),
        }
    }
    pub fn to_float_compare(&self) -> Result<FloatCC, ()> {
        match self {
            Token::Less => Ok(FloatCC::LessThan),
            Token::LessEqual => Ok(FloatCC::LessThanOrEqual),
            Token::Greater => Ok(FloatCC::GreaterThan),
            Token::GreaterEqual => Ok(FloatCC::GreaterThanOrEqual),
            Token::EqualEqual => Ok(FloatCC::Equal),
            Token::NotEqual => Ok(FloatCC::NotEqual),
            _ => Err(()),
        }
    }
    pub fn without_assignment(&self) -> Result<Token, ()> {
        Ok(match self {
            Token::PlusEqual => Token::Plus,
            Token::MinusEqual => Token::Minus,
            Token::StarEqual => Token::Star,
            Token::DivideEqual => Token::Divide,
            Token::ModEqual => Token::Mod,
            Token::AndEqual => Token::Ampersand,
            Token::OrEqual => Token::BitwiseOr,
            Token::LeftEqual => Token::ShiftLeft,
            Token::RightEqual => Token::ShiftRight,
            Token::XorEqual => Token::Xor,
            _ => return Err(()),
        })
    }
}

impl Type {
    pub fn strong_eq(&self, other: &Self, types: &Types) -> bool {
        match (self, other) {
            (Type::Void, Type::Void)
            | (Type::Bool, Type::Bool)
            | (Type::Float, Type::Float)
            | (Type::Double, Type::Double)
            | (Type::VaList, Type::VaList) => true,
            (Type::Char(x), Type::Char(y))
            | (Type::Short(x), Type::Short(y))
            | (Type::Int(x), Type::Int(y))
            | (Type::Long(x), Type::Long(y)) => x == y,
            (Type::Pointer(x_p, x_qual), Type::Pointer(y_p, y_qual)) => {
                x_p == y_p && x_qual == y_qual
            }
            (Type::Array(x_p, x_s), Type::Array(y_p, y_s)) => x_p == y_p && x_s == y_s,
            (Type::Function(x), Type::Function(y)) => x.strong_eq(y, types),
            (Type::Union(x_id, x_members), Type::Union(y_id, y_members))
            | (Type::Struct(x_id, x_members), Type::Struct(y_id, y_members)) => {
                x_id == y_id && x_members == y_members
            }
            (Type::Enum(x_id, x_members), Type::Enum(y_id, y_members)) => {
                x_id == y_id && x_members == y_members
            }
            (Type::Bitfield(x), Type::Bitfield(y)) => x == y,
            _ => false,
        }
    }
    /// https://stackoverflow.com/questions/14821936/what-is-a-scalar-object-in-c#14822074
    #[inline]
    pub fn is_scalar(&self) -> bool {
        use Type::*;
        match self {
            Enum(_, _) => true,
            k if k.is_arithmetic() || k.is_pointer() => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_bool(&self) -> bool {
        match self {
            Type::Bool => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_char(&self) -> bool {
        match self {
            Type::Char(true) => true,
            _ => false,
        }
    }
    #[inline]
    // returns whether `self` is a signed integer type
    pub fn is_signed(&self) -> bool {
        use Type::*;
        match self {
            Bool | Char(true) | Short(true) | Int(true) | Long(true) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_integral(&self) -> bool {
        use Type::*;
        match self {
            Bool | Char(_) | Short(_) | Int(_) | Long(_) | Enum(_, _) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_floating(&self) -> bool {
        match self {
            Type::Float | Type::Double => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_arithmetic(&self) -> bool {
        self.is_integral() || self.is_floating()
    }
    #[inline]
    pub fn is_pointer(&self) -> bool {
        match self {
            Type::Pointer(_, _) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_void_pointer(&self, types: &Types) -> bool {
        match self {
            Type::Pointer(t, _) => **t == Type::Void,
            _ => false,
        }
    }
    #[inline]
    /// used for pointer addition and subtraction, see section 6.5.6 of the C11 standard
    pub fn is_pointer_to_complete_object(&self, types: &Types) -> bool {
        match self {
            Type::Pointer(ctype, _) => ctype.is_complete() && !ctype.is_function(),
            _ => false,
        }
    }
    pub fn is_complete(&self) -> bool {
        match self {
            Type::Void | Type::Array(_, ArrayType::Unbounded) => false,
            // TODO: update when we allow incomplete struct and union types (e.g. `struct s;`)
            _ => true,
        }
    }
    #[inline]
    pub fn is_function(&self) -> bool {
        match self {
            Type::Function(_) => true,
            _ => false,
        }
    }
}

pub enum LengthError {
    Unbounded,
    Dynamic,
    NonIntegral,
    Negative,
}

impl Expr {
    pub fn const_int(self, types: &Types) -> SemanticResult<SIZE_T> {
        if !types[self.ctype].is_integral() {
            return Err(Locatable {
                data: LengthError::NonIntegral.into(),
                location: self.location.clone(),
            });
        }
        let literal = self.constexpr(types)?.map_err(|location| Locatable {
            data: LengthError::Dynamic.into(),
            location,
        })?;
        match literal.data.0 {
            Token::UnsignedInt(u) => Ok(u),
            Token::Int(x) => x.try_into().map_err(|_| Locatable {
                data: LengthError::Negative.into(),
                location: literal.location,
            }),
            x => unreachable!("should have been caught already: {:?}", x),
        }
    }
    pub fn zero(types: &mut Types) -> Expr {
        Expr {
            ctype: types.get_or_insert(Type::Int(true)),
            constexpr: true,
            expr: ExprType::Literal(Token::Int(0)),
            lval: false,
            location: Default::default(),
        }
    }
}

impl From<LengthError> for String {
    fn from(err: LengthError) -> String {
        let s: &'static str = err.into();
        s.to_string()
    }
}

impl From<LengthError> for &'static str {
    fn from(err: LengthError) -> &'static str {
        use LengthError::*;
        match err {
            Unbounded => "Cannot take the length of unbounded array type",
            Dynamic => "Length of variable-length array cannot be known at compile time",
            NonIntegral => "The length of an array must be an integer",
            Negative => "The length of an array must not be negative",
        }
    }
}

impl<K: Hash + Eq, V> Scope<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self(vec![HashMap::new()])
    }
    #[inline]
    pub fn enter_scope(&mut self) {
        self.0.push(HashMap::<K, V>::new())
    }
    #[inline]
    pub fn leave_scope(&mut self) {
        if self.0.len() == 1 {
            panic!("cannot leave the global scope");
        }
        self.0.pop();
    }
    pub fn get(&self, name: &K) -> Option<&V> {
        for map in self.0.iter().rev() {
            let current = map.get(name);
            if current.is_some() {
                return current;
            }
        }
        None
    }
    // returns whether the _immediate_ scope contains `name`
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.last_mut().unwrap().insert(key, value)
    }
    #[inline]
    pub fn get_immediate(&self, name: &K) -> Option<&V> {
        self.0.last().unwrap().get(name)
    }
    #[inline]
    pub fn get_all_immediate(&mut self) -> &mut HashMap<K, V> {
        self.0.last_mut().unwrap()
    }
    #[inline(always)]
    pub fn depth(&self) -> usize {
        self.0.len()
    }
    pub fn is_global(&self) -> bool {
        self.0.len() == 1
    }
}

impl<K: Eq + Hash, V> Default for Scope<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionType {
    pub fn strong_eq(&self, other: &Self, types: &Types) -> bool {
        self.params == other.params
            && types[self.return_type].strong_eq(&types[other.return_type], types)
            && self.varargs == other.varargs
    }
    pub fn should_return(&self, types: &Types) -> bool {
        types[self.return_type] != Type::Void
    }
    pub fn has_params(&self, types: &Types) -> bool {
        !(self.params.len() == 1 && types[self.params[0].ctype] == Type::Void)
    }
}

impl TryFrom<Keyword> for StorageClass {
    type Error = Keyword;
    fn try_from(value: Keyword) -> Result<StorageClass, Keyword> {
        use StorageClass::*;
        match value {
            Keyword::Extern => Ok(Extern),
            Keyword::Static => Ok(Static),
            Keyword::Auto => Ok(Auto),
            Keyword::Register => Ok(Register),
            Keyword::Typedef => Ok(Typedef),
            _ => Err(value),
        }
    }
}

impl Default for StorageClass {
    fn default() -> StorageClass {
        StorageClass::Auto
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Keyword::Alignas
            | Keyword::Alignof
            | Keyword::Bool
            | Keyword::Complex
            | Keyword::Imaginary
            | Keyword::Atomic
            | Keyword::ThreadLocal
            | Keyword::NoReturn
            | Keyword::Generic
            | Keyword::StaticAssert => write!(f, "_{:?}", self),
            Keyword::VaList => write!(f, "va_list"),
            _ => write!(f, "{}", &format!("{:?}", self).to_lowercase()),
        }
    }
}

impl Display for StorageClass {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", &format!("{:?}", self).to_lowercase())
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Type::*;
        match self {
            Char(signed) | Short(signed) | Int(signed) | Long(signed) => {
                let lower = &format!("{:?}", self).to_lowercase();
                let substr = match lower.find('(') {
                    Some(n) => &lower[..n],
                    None => lower.as_str(),
                };
                write!(f, "{}{}", if *signed { "" } else { "unsigned " }, substr)
            }
            Bool | Float | Double | Void => write!(f, "{}", format!("{:?}", self).to_lowercase()),
            Pointer(to, _) => {
                to.print_pre(f)?;
                self.print_mid(f)?;
                to.print_post(f)
            }
            Array(of, size) => {
                of.print_pre(f)?;
                of.print_mid(f)?;
                self.print_post(f)
            }
            Function(FunctionType {
                return_type,
                params,
                varargs,
            }) => {
                write!(f, "{}", return_type)?;
                self.print_post(f)
            }
            Enum(Some(ident), _) => write!(f, "enum {}", ident),
            Enum(None, members) => write!(f, "<anonymous enum>"),
            Union(Some(ident), _) => write!(f, "union {}", ident),
            Union(None, members) => write!(f, "<anonymous union>"),
            Struct(Some(ident), _) => write!(f, "struct {}", ident),
            Struct(None, members) => write!(f, "<anonymous struct>"),
            Bitfield(_) => unimplemented!("printing bitfield type"),
            VaList => write!(f, "va_list"),
        }
    }
}
impl Type {
    fn print_pre(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Pointer(t, _) | Type::Array(t, _) => t.print_pre(f),
            Type::Function(func_type) => Display::fmt(&func_type.return_type, f),
            _ => write!(f, "{}", self),
        }
    }
    fn print_mid(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Pointer(to, quals) => {
                to.print_mid(f)?;
                let ptr_description = match (quals.c_const, quals.volatile) {
                    (true, true) => "const volatile ",
                    (true, false) => "const ",
                    (false, true) => "volatile ",
                    (false, false) => "",
                };
                match &**to {
                    Type::Array(_, _) | Type::Function(_) => write!(f, "(*{})", ptr_description),
                    _ => write!(f, " *{}", ptr_description),
                }
            }
            Type::Array(to, _) => to.print_mid(f),
            _ => Ok(()),
        }
    }
    fn print_post(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Array(to, size) => {
                write!(f, "[")?;
                if let ArrayType::Fixed(size) = size {
                    write!(f, "{}", size)?;
                }
                write!(f, "]")?;
                to.print_post(f)
            }
            Type::Function(func_type) => {
                // https://stackoverflow.com/a/30325430
                let mut comma_seperated = "(".to_string();
                for param in &func_type.params {
                    comma_seperated.push_str(&param.ctype.to_string());
                    comma_seperated.push_str(", ");
                }
                if func_type.varargs {
                    comma_seperated.push_str("...");
                } else if !func_type.params.is_empty() {
                    comma_seperated.pop();
                    comma_seperated.pop();
                }
                comma_seperated.push(')');
                write!(f, "{}", comma_seperated)
            }
            _ => Ok(()),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Token::*;
        match self {
            PlusPlus => write!(f, "++"),
            PlusEqual => write!(f, "+="),
            MinusMinus => write!(f, "--"),
            MinusEqual => write!(f, "+="),
            StarEqual => write!(f, "*="),
            DivideEqual => write!(f, "/="),
            ModEqual => write!(f, "%="),
            AndEqual => write!(f, "&="),
            OrEqual => write!(f, "|="),
            XorEqual => write!(f, "^="),
            LeftEqual => write!(f, "<<="),
            RightEqual => write!(f, ">>="),
            EqualEqual => write!(f, "=="),
            NotEqual => write!(f, "!="),
            LessEqual => write!(f, "<="),
            GreaterEqual => write!(f, ">="),
            ShiftRight => write!(f, ">>"),
            ShiftLeft => write!(f, "<<"),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Star => write!(f, "*"),
            Divide => write!(f, "/"),
            Xor => write!(f, "^"),
            Equal => write!(f, "="),
            Less => write!(f, "<"),
            Greater => write!(f, ">"),
            Ampersand => write!(f, "&"),
            LogicalAnd => write!(f, "&&"),
            BitwiseOr => write!(f, "|"),
            LogicalOr => write!(f, "||"),
            BinaryNot => write!(f, "~"),
            LogicalNot => write!(f, "!"),
            LeftBrace => write!(f, "{{"),
            RightBrace => write!(f, "}}"),
            LeftBracket => write!(f, "["),
            RightBracket => write!(f, "]"),
            LeftParen => write!(f, "("),
            RightParen => write!(f, ")"),
            Semicolon => write!(f, ";"),
            Colon => write!(f, ":"),
            Comma => write!(f, ","),
            Dot => write!(f, "."),
            Question => write!(f, "?"),
            Mod => write!(f, "%"),

            Int(i) => write!(f, "{}", i),
            UnsignedInt(u) => write!(f, "{}", u),
            Float(n) => write!(f, "{}", n),
            Str(s) => write!(f, "\"{}\"", s),
            Char(c) => write!(f, "{}", c),
            Id(id) => write!(f, "{}", id),
            Keyword(k) => write!(f, "{}", k),

            Ellipsis => write!(f, "..."),
            StructDeref => write!(f, "->"),
        }
    }
}

impl Display for Qualifiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match (self.c_const, self.volatile) {
                (true, true) => "'const volatile' type qualifiers",
                (true, false) => "'const' type qualifier",
                (false, true) => "'volatile' type qualifier",
                (false, false) => "",
            }
        )
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.expr {
            ExprType::Comma(left, right) => write!(f, "{:?}, {:?}", *left, *right),
            ExprType::Literal(token) => write!(f, "{}", token),
            ExprType::Id(symbol) => write!(f, "{}", symbol.id),
            ExprType::Add(left, right) => write!(f, "({:?}) + ({:?})", left, right),
            ExprType::Sub(left, right) => write!(f, "({:?}) - ({:?})", left, right),
            ExprType::Mul(left, right) => write!(f, "({:?}) * ({:?})", left, right),
            ExprType::Div(left, right) => write!(f, "({:?}) / ({:?})", left, right),
            ExprType::Mod(left, right) => write!(f, "({:?}) % ({:?})", left, right),
            ExprType::Xor(left, right) => write!(f, "({:?}) ^ ({:?})", left, right),
            ExprType::BitwiseOr(left, right) => write!(f, "({:?}) | ({:?})", left, right),
            ExprType::BitwiseAnd(left, right) => write!(f, "({:?}) & ({:?})", left, right),
            ExprType::BitwiseNot(expr) => write!(f, "(~{:?})", expr),
            ExprType::Deref(expr) => write!(f, "*({:?})", expr),
            ExprType::Negate(expr) => write!(f, "-({:?})", expr),
            ExprType::LogicalNot(expr) => write!(f, "!({:?})", expr),
            ExprType::LogicalOr(left, right) => write!(f, "({:?}) || ({:?})", left, right),
            ExprType::LogicalAnd(left, right) => write!(f, "({:?}) && ({:?})", left, right),
            ExprType::Shift(val, by, left) => write!(
                f,
                "({:?}) {} ({:?})",
                val,
                if *left { "<<" } else { ">>" },
                by
            ),
            ExprType::Compare(left, right, token) => {
                write!(f, "({:?}) {} ({:?})", left, token, right)
            }
            ExprType::Assign(left, right, token) => {
                write!(f, "({:?}) {} ({:?})", left, token, right)
            }
            ExprType::Ternary(cond, left, right) => {
                write!(f, "({:?}) ? ({:?}) : ({:?})", cond, left, right)
            }
            ExprType::FuncCall(left, params) => write!(
                f,
                "({:?})({})",
                left,
                print_func_call(params.as_slice(), |expr| {
                    let mut s = String::new();
                    write!(s, "{:?}", expr).unwrap();
                    s
                })
            ),
            ExprType::Cast(expr) => write!(f, "({})({:?})", self.ctype, expr),
            ExprType::Sizeof(ty) => write!(f, "sizeof({})", ty),
            ExprType::Member(compound, id) => write!(f, "({:?}).{}", compound, id),
            ExprType::Increment(expr, pre, inc) => unimplemented!("printing increments"),
        }
    }
}

fn print_func_call<T, F: Fn(&T) -> String>(params: &[T], print_func: F) -> String {
    // https://stackoverflow.com/a/30325430
    let mut comma_separated = String::new();
    for param in params {
        comma_separated.push_str(&print_func(param));
        comma_separated.push_str(", ");
    }
    comma_separated.pop();
    comma_separated.pop();
    comma_separated
}

impl Debug for Initializer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Initializer::Scalar(expr) => write!(f, "{:?};", expr),
            Initializer::InitializerList(list) => {
                write!(f, "{{ ")?;
                write!(
                    f,
                    "{}",
                    print_func_call(list, |init| { format!("{:?}", init) })
                )?;
                write!(f, " }};")
            }
            Initializer::FunctionBody(body) => {
                writeln!(f, "{{")?;
                for stmt in body {
                    writeln!(f, "{:?}", stmt.data)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Debug for StmtType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StmtType::Expr(expr) => write!(f, "{:?};", expr),
            StmtType::Return(None) => write!(f, "return;"),
            StmtType::Return(Some(expr)) => write!(f, "return {:?};", expr),
            _ => unimplemented!("printing statement"),
        }
    }
}

impl PartialEq for ArrayType {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for ArrayType {}

impl PartialEq for Symbol {
    // don't require both symbols to be `init` to be equal
    fn eq(&self, other: &Self) -> bool {
        self.ctype == other.ctype
            && self.id == other.id
            && self.qualifiers == other.qualifiers
            && self.storage_class == other.storage_class
    }
}

impl PartialEq for FunctionType {
    fn eq(&self, other: &Self) -> bool {
        // no prototype: any parameters are allowed
        // TODO: issue a warning if a function has empty parameters, it's a holdover
        // from C89
        self.params.is_empty()
            || other.params.is_empty()
            || self.varargs == other.varargs
            && self.return_type == other.return_type
            // don't require parameter names and storage_class to match
            && self.params
                .iter()
                .zip(other.params.iter())
                .all(|(this_param, other_param)| {
                    this_param.ctype == other_param.ctype
                        && this_param.qualifiers == other_param.qualifiers
                })
    }
}

impl Eq for Symbol {}

mod tests {
    #[test]
    fn type_display() {
        for ty in [
            "int",
            "int *",
            "int[1][2][3]",
            "int *(*)(int)",
            "int *(*)[1][2][3]",
        ]
        .iter()
        {
            assert_eq!(
                &format!(
                    "{}",
                    crate::Parser::new(
                        crate::Lexer::new("<integration-test>".into(), ty.chars(), false),
                        false
                    )
                    .type_name()
                    .unwrap()
                    .data
                    .0
                ),
                ty
            );
        }
    }
}
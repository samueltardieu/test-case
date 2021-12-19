use crate::utils::fmt_syn;
use proc_macro2::TokenStream;
use quote::quote;
use std::fmt::{Display, Formatter};
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr};

mod kw {
    syn::custom_keyword!(eq);
    syn::custom_keyword!(equal_to);
    syn::custom_keyword!(lt);
    syn::custom_keyword!(less_than);
    syn::custom_keyword!(gt);
    syn::custom_keyword!(greater_than);
    syn::custom_keyword!(leq);
    syn::custom_keyword!(less_or_equal_than);
    syn::custom_keyword!(geq);
    syn::custom_keyword!(greater_or_equal_than);
    syn::custom_keyword!(almost);
    syn::custom_keyword!(almost_equal_to);
    syn::custom_keyword!(precision);
    syn::custom_keyword!(existing_path);
    syn::custom_keyword!(directory);
    syn::custom_keyword!(dir);
    syn::custom_keyword!(file);
    syn::custom_keyword!(contains);
    syn::custom_keyword!(contains_in_order);
}

#[derive(Debug, PartialEq)]
pub enum OrderingToken {
    Eq,
    Lt,
    Gt,
    Leq,
    Geq,
}

#[derive(Debug, PartialEq)]
pub enum PathToken {
    Any,
    Dir,
    File,
}

#[derive(Debug, PartialEq)]
pub struct Ord {
    pub token: OrderingToken,
    pub expected_value: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct AlmostEqual {
    pub expected_value: Box<Expr>,
    pub precision: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub token: PathToken,
}

#[derive(Debug, PartialEq)]
pub struct Contains {
    pub expected_element: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct ContainsInOrder {
    pub expected_slice: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub enum ComplexTestCase {
    // Not(Box<ComplexTestCase>),
    // And(Vec<ComplexTestCase>),
    // Or(Vec<ComplexTestCase>),
    Ord(Ord),
    AlmostEqual(AlmostEqual),
    Path(Path),
    Contains(Contains),
    ContainsInOrder(ContainsInOrder),
}

impl Parse for ComplexTestCase {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<kw::eq>().is_ok() || input.parse::<kw::equal_to>().is_ok() {
            Ok(ComplexTestCase::Ord(Ord {
                token: OrderingToken::Eq,
                expected_value: input.parse()?,
            }))
        } else if input.parse::<kw::lt>().is_ok() || input.parse::<kw::less_than>().is_ok() {
            Ok(ComplexTestCase::Ord(Ord {
                token: OrderingToken::Lt,
                expected_value: input.parse()?,
            }))
        } else if input.parse::<kw::gt>().is_ok() || input.parse::<kw::greater_than>().is_ok() {
            Ok(ComplexTestCase::Ord(Ord {
                token: OrderingToken::Gt,
                expected_value: input.parse()?,
            }))
        } else if input.parse::<kw::leq>().is_ok()
            || input.parse::<kw::less_or_equal_than>().is_ok()
        {
            Ok(ComplexTestCase::Ord(Ord {
                token: OrderingToken::Leq,
                expected_value: input.parse()?,
            }))
        } else if input.parse::<kw::geq>().is_ok()
            || input.parse::<kw::greater_or_equal_than>().is_ok()
        {
            Ok(ComplexTestCase::Ord(Ord {
                token: OrderingToken::Geq,
                expected_value: input.parse()?,
            }))
        } else if input.parse::<kw::almost>().is_ok()
            || input.parse::<kw::almost_equal_to>().is_ok()
        {
            let target = input.parse()?;
            let _ = input.parse::<kw::precision>()?;
            let precision = input.parse()?;
            Ok(ComplexTestCase::AlmostEqual(AlmostEqual {
                expected_value: target,
                precision: precision,
            }))
        } else if input.parse::<kw::existing_path>().is_ok() {
            Ok(ComplexTestCase::Path(Path {
                token: PathToken::Any,
            }))
        } else if input.parse::<kw::directory>().is_ok() || input.parse::<kw::dir>().is_ok() {
            Ok(ComplexTestCase::Path(Path {
                token: PathToken::Dir,
            }))
        } else if input.parse::<kw::file>().is_ok() {
            Ok(ComplexTestCase::Path(Path {
                token: PathToken::File,
            }))
        } else if input.parse::<kw::contains>().is_ok() {
            Ok(ComplexTestCase::Contains(Contains {
                expected_element: input.parse()?,
            }))
        } else if input.parse::<kw::contains_in_order>().is_ok() {
            Ok(ComplexTestCase::ContainsInOrder(ContainsInOrder {
                expected_slice: input.parse()?,
            }))
        } else {
            proc_macro_error::abort!(input.span(), "cannot parse complex expression")
        }
    }
}

impl Display for OrderingToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderingToken::Eq => f.write_str("eq"),
            OrderingToken::Lt => f.write_str("lt"),
            OrderingToken::Gt => f.write_str("gt"),
            OrderingToken::Leq => f.write_str("leq"),
            OrderingToken::Geq => f.write_str("geq"),
        }
    }
}

impl Display for PathToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathToken::Any => f.write_str("path"),
            PathToken::Dir => f.write_str("dir"),
            PathToken::File => f.write_str("file"),
        }
    }
}

impl Display for ComplexTestCase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexTestCase::Ord(Ord {
                token: ord,
                expected_value: expr,
            }) => write!(f, "{} {}", ord, fmt_syn(expr)),
            ComplexTestCase::AlmostEqual(AlmostEqual {
                expected_value: target,
                precision,
            }) => {
                write!(f, "almost {} p {}", fmt_syn(target), fmt_syn(precision))
            }
            ComplexTestCase::Path(Path { token }) => write!(f, "{}", token),
            ComplexTestCase::Contains(Contains {
                expected_element: elem,
            }) => write!(f, "{}", fmt_syn(elem)),
            ComplexTestCase::ContainsInOrder(ContainsInOrder {
                expected_slice: elems,
            }) => write!(f, "{}", fmt_syn(elems)),
        }
    }
}

impl ComplexTestCase {
    pub fn assertion(&self) -> TokenStream {
        match self {
            ComplexTestCase::Ord(Ord {
                token: ord,
                expected_value: expr,
            }) => ord_assertion(ord, expr),
            ComplexTestCase::AlmostEqual(AlmostEqual {
                expected_value: expr,
                precision,
            }) => almost_equal_assertion(expr, precision),
            ComplexTestCase::Path(Path { token: kind }) => path_assertion(kind),
            ComplexTestCase::Contains(Contains {
                expected_element: element,
            }) => contains_assertion(element),
            ComplexTestCase::ContainsInOrder(ContainsInOrder {
                expected_slice: elements,
            }) => contains_in_order_assertion(elements),
        }
    }
}

fn contains_in_order_assertion(elements: &Expr) -> TokenStream {
    parse_quote! {
        let mut _tc_outcome = false;
        for i in 0..=_result.len() - #elements.len() {
            if #elements == _result[i..i+#elements.len()] {
                _tc_outcome = true;
            }
        }
        assert!(_tc_outcome, "contains_in_order failed")
    }
}

fn contains_assertion(element: &Expr) -> TokenStream {
    parse_quote! { assert!(_result.iter().find(|i| i.eq(&&#element)).is_some()) }
}

fn path_assertion(token: &PathToken) -> TokenStream {
    match token {
        PathToken::Any => parse_quote! { assert!(std::path::Path::new(&_result).exists()) },
        PathToken::Dir => parse_quote! { assert!(std::path::Path::new(&_result).is_dir()) },
        PathToken::File => parse_quote! { assert!(std::path::Path::new(&_result).is_file()) },
    }
}

fn almost_equal_assertion(expr: &Expr, precision: &Expr) -> TokenStream {
    quote! { assert!((_result - #expr).abs() < #precision) }
}

fn ord_assertion(ord: &OrderingToken, expr: &Expr) -> TokenStream {
    let ts: TokenStream = match ord {
        OrderingToken::Eq => parse_quote! { == },
        OrderingToken::Lt => parse_quote! { < },
        OrderingToken::Gt => parse_quote! { > },
        OrderingToken::Leq => parse_quote! { <= },
        OrderingToken::Geq => parse_quote! { >= },
    };

    quote! {
        assert!(_result #ts #expr)
    }
}

#[cfg(test)]
mod tests {
    use crate::complex_expr::{
        AlmostEqual, ComplexTestCase, Contains, ContainsInOrder, OrderingToken, Path, PathToken,
    };
    use syn::{parse_quote, LitFloat, LitInt, LitStr};

    macro_rules! assert_ord {
        ($actual:tt, $token:path, $value:tt) => {
            if let ComplexTestCase::Ord(ord) = $actual {
                assert_eq!(ord.token, $token);
                let lit = ord.expected_value;
                let actual_expr: LitFloat = parse_quote! { #lit };
                assert_eq!(actual_expr.base10_parse::<f64>().unwrap(), $value)
            } else {
                panic!("invalid enum variant")
            }
        };
    }

    macro_rules! assert_almost_eq {
        ($actual:tt, $value:tt, $precision:tt) => {
            if let ComplexTestCase::AlmostEqual(AlmostEqual {
                expected_value,
                precision,
            }) = $actual
            {
                let expected_value: LitFloat = parse_quote! { #expected_value };
                assert_eq!(expected_value.base10_parse::<f64>().unwrap(), $value);
                let precision: LitFloat = parse_quote! { #precision };
                assert_eq!(precision.base10_parse::<f64>().unwrap(), $precision);
            } else {
                panic!("invalid enum variant")
            }
        };
    }

    #[test]
    fn parses_ord_token_stream() {
        let actual: ComplexTestCase = parse_quote! { equal_to 1.0 };
        assert_ord!(actual, OrderingToken::Eq, 1.0);
        let actual: ComplexTestCase = parse_quote! { eq 0.0 };
        assert_ord!(actual, OrderingToken::Eq, 0.0);

        let actual: ComplexTestCase = parse_quote! { less_than 2.0 };
        assert_ord!(actual, OrderingToken::Lt, 2.0);
        let actual: ComplexTestCase = parse_quote! { lt 2.0 };
        assert_ord!(actual, OrderingToken::Lt, 2.0);

        let actual: ComplexTestCase = parse_quote! { greater_than 2.0 };
        assert_ord!(actual, OrderingToken::Gt, 2.0);
        let actual: ComplexTestCase = parse_quote! { gt 2.0 };
        assert_ord!(actual, OrderingToken::Gt, 2.0);

        let actual: ComplexTestCase = parse_quote! { less_or_equal_than 2.0 };
        assert_ord!(actual, OrderingToken::Leq, 2.0);
        let actual: ComplexTestCase = parse_quote! { leq 2.0 };
        assert_ord!(actual, OrderingToken::Leq, 2.0);

        let actual: ComplexTestCase = parse_quote! { greater_or_equal_than 2.0 };
        assert_ord!(actual, OrderingToken::Geq, 2.0);
        let actual: ComplexTestCase = parse_quote! { geq 2.0 };
        assert_ord!(actual, OrderingToken::Geq, 2.0);
    }

    #[test]
    fn can_parse_eq_other_types() {
        let actual: ComplexTestCase = parse_quote! { equal_to "abcde" };
        if let ComplexTestCase::Ord(ord) = actual {
            assert_eq!(ord.token, OrderingToken::Eq);
            let lit = ord.expected_value;
            let actual_expr: LitStr = parse_quote! { #lit };
            assert_eq!(actual_expr.value(), "abcde")
        } else {
            panic!("invalid enum variant")
        }

        let actual: ComplexTestCase = parse_quote! { equal_to 1 };
        if let ComplexTestCase::Ord(ord) = actual {
            assert_eq!(ord.token, OrderingToken::Eq);
            let lit = ord.expected_value;
            let actual_expr: LitInt = parse_quote! { #lit };
            assert_eq!(actual_expr.base10_parse::<i64>().unwrap(), 1)
        } else {
            panic!("invalid enum variant")
        }
    }

    #[test]
    fn parses_almost_equal_token_stream() {
        let actual: ComplexTestCase = parse_quote! { almost_equal_to 1.0 precision 0.1 };
        assert_almost_eq!(actual, 1.0, 0.1);
        let actual: ComplexTestCase = parse_quote! { almost_equal_to 1.0 precision 0.0f32 };
        assert_almost_eq!(actual, 1.0, 0.0);
    }

    #[test]
    fn parses_path_token_stream() {
        let actual: ComplexTestCase = parse_quote! { existing_path };
        assert_eq!(
            actual,
            ComplexTestCase::Path(Path {
                token: PathToken::Any
            })
        );
        let actual: ComplexTestCase = parse_quote! { file };
        assert_eq!(
            actual,
            ComplexTestCase::Path(Path {
                token: PathToken::File
            })
        );
        let actual: ComplexTestCase = parse_quote! { dir };
        assert_eq!(
            actual,
            ComplexTestCase::Path(Path {
                token: PathToken::Dir
            })
        );
        let actual: ComplexTestCase = parse_quote! { directory };
        assert_eq!(
            actual,
            ComplexTestCase::Path(Path {
                token: PathToken::Dir
            })
        );
    }

    #[test]
    fn parses_contains_token_stream() {
        let actual: ComplexTestCase = parse_quote! { contains 1.0 };
        assert_eq!(
            actual,
            ComplexTestCase::Contains(Contains {
                expected_element: Box::new(parse_quote! { 1.0 })
            })
        );
        let actual: ComplexTestCase = parse_quote! { contains "abcde" };
        assert_eq!(
            actual,
            ComplexTestCase::Contains(Contains {
                expected_element: Box::new(parse_quote! { "abcde" })
            })
        );
        let actual: ComplexTestCase = parse_quote! { contains true };
        assert_eq!(
            actual,
            ComplexTestCase::Contains(Contains {
                expected_element: Box::new(parse_quote! { true })
            })
        );
    }

    #[test]
    fn parses_contains_in_order_token_stream() {
        let actual: ComplexTestCase = parse_quote! { contains_in_order [1, 2, 3] };
        assert_eq!(
            actual,
            ComplexTestCase::ContainsInOrder(ContainsInOrder {
                expected_slice: Box::new(parse_quote! { [1, 2, 3] })
            })
        )
    }
}

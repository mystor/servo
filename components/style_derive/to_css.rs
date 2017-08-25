/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote;
use syn;
use synstructure;

pub fn derive(input: synstructure::Structure) -> quote::Tokens {
    let match_body = input.each_variant(|v| {
        let mut identifier = to_css_identifier(v.ast().ident.as_ref());
        let mut css_attrs = v.ast().attrs.iter().filter(|attr| attr.name() == "css");
        let (is_function, use_comma) = css_attrs.next().map_or((false, false), |attr| {
            match attr.value {
                syn::MetaItem::List(ref ident, ref items) if ident.as_ref() == "css" => {
                    let mut nested = items.iter();
                    let mut is_function = false;
                    let mut use_comma = false;
                    for attr in nested.by_ref() {
                        match *attr {
                            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) => {
                                match ident.as_ref() {
                                    "function" => {
                                        if is_function {
                                            panic!("repeated `#[css(function)]` attribute");
                                        }
                                        is_function = true;
                                    },
                                    "comma" => {
                                        if use_comma {
                                            panic!("repeated `#[css(comma)]` attribute");
                                        }
                                        use_comma = true;
                                    },
                                    _ => panic!("only `#[css(function | comma)]` is supported for now"),
                                }
                            },
                            _ => panic!("only `#[css(<ident...>)]` is supported for now"),
                        }
                    }
                    if nested.next().is_some() {
                        panic!("only `#[css()]` or `#[css(<ident>)]` is supported for now")
                    }
                    (is_function, use_comma)
                },
                _ => panic!("only `#[css(...)]` is supported for now"),
            }
        });
        if css_attrs.next().is_some() {
            panic!("only a single `#[css(...)]` attribute is supported for now");
        }
        let separator = if use_comma { ", " } else { " " };
        let mut expr = if !v.bindings().is_empty() {
            let mut expr = quote! {};
            for binding in v.bindings() {
                expr = quote! {
                    #expr
                    writer.item(#binding)?;
                };
            }
            quote! {{
                let mut writer = ::style_traits::values::SequenceWriter::new(&mut *dest, #separator);
                #expr
                Ok(())
            }}
        } else {
            quote! {
                ::std::fmt::Write::write_str(dest, #identifier)
            }
        };
        if is_function {
            identifier.push_str("(");
            expr = quote! {
                ::std::fmt::Write::write_str(dest, #identifier)?;
                #expr?;
                ::std::fmt::Write::write_str(dest, ")")
            }
        }
        expr
    });

    input.bound_impl("::style_traits::ToCss", quote! {
        #[allow(unused_variables)]
        #[inline]
        fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where
            W: ::std::fmt::Write
        {
            match *self {
                #match_body
            }
        }
    })
}

/// Transforms "FooBar" to "foo-bar".
///
/// If the first Camel segment is "Moz" or "Webkit", the result string
/// is prepended with "-".
fn to_css_identifier(mut camel_case: &str) -> String {
    camel_case = camel_case.trim_right_matches('_');
    let mut first = true;
    let mut result = String::with_capacity(camel_case.len());
    while let Some(segment) = split_camel_segment(&mut camel_case) {
        if first {
            match segment {
                "Moz" | "Webkit" => first = false,
                _ => {},
            }
        }
        if !first {
            result.push_str("-");
        }
        first = false;
        result.push_str(&segment.to_lowercase());
    }
    result
}

/// Given "FooBar", returns "Foo" and sets `camel_case` to "Bar".
fn split_camel_segment<'input>(camel_case: &mut &'input str) -> Option<&'input str> {
    let index = match camel_case.chars().next() {
        None => return None,
        Some(ch) => ch.len_utf8(),
    };
    let end_position = camel_case[index..]
        .find(char::is_uppercase)
        .map_or(camel_case.len(), |pos| index + pos);
    let result = &camel_case[..end_position];
    *camel_case = &camel_case[end_position..];
    Some(result)
}

#[cfg(test)]
mod test {
    #[test]
    fn simple() {
        test_derive! {
            super::derive {
                enum A<T> {
                    SomeThing(T, Option<T>),
                    UnitVariant,
                    MozUnitVariant,
                    WebkitUnitVariant,
                    #[css(function)]
                    FuncThing(T, Option<T>),
                    #[css(function, comma)]
                    FuncCommaThing(T, Option<T>),
                }
            }
            expands to {
                impl<T> ::style_traits::ToCss for A<T>
                where
                    T: ::style_traits::ToCss
                {
                    #[allow(unused_variables)]
                    #[inline]
                    fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                    where
                        W: ::std::fmt::Write
                    {
                        match *self {
                            A::SomeThing(ref __binding_0, ref __binding_1,) => {
                                {
                                    let mut writer = ::style_traits::values::SequenceWriter::new(
                                        &mut *dest,
                                        " "
                                    );
                                    writer.item(__binding_0)?;
                                    writer.item(__binding_1)?;
                                    Ok(())
                                }
                            }
                            A::UnitVariant => {
                                ::std::fmt::Write::write_str(dest, "unit-variant")
                            }
                            A::MozUnitVariant => {
                                ::std::fmt::Write::write_str(dest, "-moz-unit-variant")
                            }
                            A::WebkitUnitVariant => {
                                ::std::fmt::Write::write_str(dest, "-webkit-unit-variant")
                            }
                            A::FuncThing(ref __binding_0, ref __binding_1,) => {
                                ::std::fmt::Write::write_str(dest, "func-thing(")?;
                                {
                                    let mut writer = ::style_traits::values::SequenceWriter::new(
                                        &mut *dest,
                                        " "
                                    );
                                    writer.item(__binding_0)?;
                                    writer.item(__binding_1)?;
                                    Ok(())
                                }?;
                                ::std::fmt::Write::write_str(dest, ")")
                            }
                            A::FuncCommaThing(ref __binding_0, ref __binding_1,) => {
                                ::std::fmt::Write::write_str(dest, "func-comma-thing(")?;
                                {
                                    let mut writer = ::style_traits::values::SequenceWriter::new(
                                        &mut *dest,
                                        ", "
                                    );
                                    writer.item(__binding_0)?;
                                    writer.item(__binding_1)?;
                                    Ok(())
                                }?;
                                ::std::fmt::Write::write_str(dest, ")")
                            }
                        }
                    }
                }
            } no_build
        }
    }

    #[test]
    fn single_function_struct() {
        test_derive! {
            super::derive {
                #[css(function)]
                struct FuncThing<T>(T, Option<T>);
            }
            expands to {
                impl<T> ::style_traits::ToCss for FuncThing<T>
                where
                    T: ::style_traits::ToCss
                {
                    #[allow(unused_variables)]
                    #[inline]
                    fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                    where
                        W: ::std::fmt::Write
                    {
                        match *self {
                            FuncThing(ref __binding_0, ref __binding_1,) => {
                                ::std::fmt::Write::write_str(dest, "func-thing(")?;
                                {
                                    let mut writer = ::style_traits::values::SequenceWriter::new(
                                        &mut *dest,
                                        " "
                                    );
                                    writer.item(__binding_0)?;
                                    writer.item(__binding_1)?;
                                    Ok(())
                                }?;
                                ::std::fmt::Write::write_str(dest, ")")
                            }
                        }
                    }
                }
            } no_build
        }
    }

    #[test]
    fn empty_struct() {
        test_derive! {
            super::derive {
                struct MozThing;
            }
            expands to {
                impl ::style_traits::ToCss for MozThing {
                    #[allow(unused_variables)]
                    #[inline]
                    fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                    where
                        W: ::std::fmt::Write
                    {
                        match *self {
                            MozThing => {
                                ::std::fmt::Write::write_str(dest, "-moz-thing")
                            }
                        }
                    }
                }
            } no_build
        }
    }
}

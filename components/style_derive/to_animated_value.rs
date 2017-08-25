/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote;
use syn;
use synstructure;

pub fn derive(mut input: synstructure::Structure) -> quote::Tokens {
    input.bind_with(|_| synstructure::BindStyle::Move);

    let animated_value_type = syn::Path::from(syn::PathSegment {
        ident: input.ast().ident.clone(),
        parameters: syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData {
            lifetimes: input.ast().generics.lifetimes.iter().map(|l| {
                l.lifetime.clone()
            }).collect(),
            types: input.ast().generics.ty_params.iter().map(|ty| {
                syn::Ty::Path(
                    Some(syn::QSelf {
                        ty: Box::new(syn::Ty::Path(None, ty.ident.clone().into())),
                        position: 3,
                    }),
                    syn::Path {
                        global: true,
                        segments: vec![
                            "values".into(),
                            "animated".into(),
                            "ToAnimatedValue".into(),
                            "AnimatedValue".into(),
                        ],
                    },
                )
            }).collect(),
            .. Default::default()
        }),
    });

    let to_body = match_body(&input, |field| {
        quote!(::values::animated::ToAnimatedValue::to_animated_value(#field))
    });
    let from_body = match_body(&input, |field| {
        quote!(::values::animated::ToAnimatedValue::from_animated_value(#field))
    });

    input.bound_impl("::values::animated::ToAnimatedValue", quote! {
        type AnimatedValue = #animated_value_type;

        #[allow(unused_variables)]
        #[inline]
        fn to_animated_value(self) -> Self::AnimatedValue {
            match self {
                #to_body
            }
        }

        #[inline]
        fn from_animated_value(animated: Self::AnimatedValue) -> Self {
            match animated {
                #from_body
            }
        }
    })
}

fn match_body<F>(input: &synstructure::Structure, f: F) -> quote::Tokens
where
    F: Fn(&synstructure::BindingInfo) -> quote::Tokens,
{
    input.each_variant(|v| {
        let mut computations = quote!();
        computations.append_all(v.bindings().iter().map(|bi| {
            let expr = f(bi);
            quote!(let #bi = #expr;)
        }));

        // NOTE: Abuse? Abuse the fact that move patterns are symmetrical to
        // construction patterns, to perform a transformation on each element,
        // and then re-construct the resulting value.
        // XXX: Add a tool for doing this to synstructure?
        let animated_value = v.pat();
        quote! {
            #computations
            #animated_value
        }
    })
}

#[cfg(test)]
mod test {
    #[test]
    fn simple() {
        test_derive! {
            super::derive {
                struct A<T> {
                    a: T,
                    b: Option<T>,
                }
            }
            expands to {
                impl<T> ::values::animated::ToAnimatedValue for A<T>
                where
                    T: ::values::animated::ToAnimatedValue
                {
                    type AnimatedValue = A<
                        <T as ::values::animated::ToAnimatedValue>::AnimatedValue
                    >;

                    #[allow(unused_variables)]
                    #[inline]
                    fn to_animated_value(self) -> Self::AnimatedValue {
                        match self {
                            A { a: __binding_0, b: __binding_1, } => {
                                let __binding_0 =
                                    ::values::animated::ToAnimatedValue::to_animated_value(
                                        __binding_0
                                    );
                                let __binding_1 =
                                    ::values::animated::ToAnimatedValue::to_animated_value(
                                        __binding_1
                                    );
                                A { a: __binding_0, b: __binding_1, }
                            }
                        }
                    }

                    #[inline]
                    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
                        match animated {
                            A { a: __binding_0, b: __binding_1, } => {
                                let __binding_0 =
                                    ::values::animated::ToAnimatedValue::from_animated_value(
                                        __binding_0
                                    );
                                let __binding_1 =
                                    ::values::animated::ToAnimatedValue::from_animated_value(
                                        __binding_1
                                    );
                                A { a: __binding_0, b: __binding_1, }
                            }
                        }
                    }
                }
            } no_build
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote;
use syn;
use synstructure;

pub fn derive(input: synstructure::Structure) -> quote::Tokens {
    let computed_value_type = syn::Path::from(syn::PathSegment {
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
                            "computed".into(),
                            "ToComputedValue".into(),
                            "ComputedValue".into(),
                        ],
                    },
                )
            }).collect(),
            .. Default::default()
        }),
    });

    let to_body = match_body(&input, |field| {
        quote!(::values::computed::ToComputedValue::to_computed_value(#field, context))
    });
    let from_body = match_body(&input, |field| {
        quote!(::values::computed::ToComputedValue::from_computed_value(#field))
    });

    input.bound_impl("::values::computed::ToComputedValue", quote! {
        type ComputedValue = #computed_value_type;

        #[allow(unused_variables)]
        #[inline]
        fn to_computed_value(&self, context: &::values::computed::Context) -> Self::ComputedValue {
            match *self {
                #to_body
            }
        }

        #[inline]
        fn from_computed_value(computed: &Self::ComputedValue) -> Self {
            match *computed {
                #from_body
            }
        }
    })
}

fn match_body<F>(input: &synstructure::Structure, f: F) -> quote::Tokens
    where F: Fn(&synstructure::BindingInfo) -> quote::Tokens,
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
        let mut move_variant = v.clone();
        move_variant.bind_with(|_| synstructure::BindStyle::Move);
        let computed_value = move_variant.pat();
        quote! {
            #computations
            #computed_value
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
                impl<T> ::values::computed::ToComputedValue for A<T>
                where
                    T: ::values::computed::ToComputedValue
                {
                    type ComputedValue = A<
                        <T as ::values::computed::ToComputedValue>::ComputedValue
                    >;

                    #[allow(unused_variables)]
                    #[inline]
                    fn to_computed_value(
                        &self,
                        context: &::values::computed::Context
                    ) -> Self::ComputedValue {
                        match *self {
                            A { a: ref __binding_0, b: ref __binding_1, } => {
                                let __binding_0 =
                                    ::values::computed::ToComputedValue::to_computed_value(
                                        __binding_0,
                                        context
                                    );
                                let __binding_1 =
                                    ::values::computed::ToComputedValue::to_computed_value(
                                        __binding_1,
                                        context
                                    );
                                A { a: __binding_0, b: __binding_1, }
                            }
                        }
                    }

                    #[inline]
                    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
                        match *computed {
                            A { a: ref __binding_0, b: ref __binding_1, } => {
                                let __binding_0 =
                                    ::values::computed::ToComputedValue::from_computed_value(
                                        __binding_0
                                    );
                                let __binding_1 =
                                    ::values::computed::ToComputedValue::from_computed_value(
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

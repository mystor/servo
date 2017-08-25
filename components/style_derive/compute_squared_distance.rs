/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote;
use synstructure;

pub fn derive(mut input: synstructure::Structure) -> quote::Tokens {
    let mut match_body = quote!();
    match_body.append_all(input.variants_mut().iter_mut().map(|variant| {
        // Make another copy of the variant to match against `other` with.
        let mut other_variant = variant.clone();
        other_variant.binding_name(|_, i| format!("other_{}", i).into());

        // Generate the body of the expression.
        let sum = if variant.bindings().is_empty() {
            quote! { ::values::distance::SquaredDistance::Value(0.) }
        } else {
            let mut sum = quote!();
            sum.append_separated(
                variant.bindings()
                    .iter()
                    .zip(other_variant.bindings())
                    .map(|(this, other)| {
                        quote! {
                            ::values::distance::ComputeSquaredDistance::compute_squared_distance(
                                #this,
                                #other,
                            )?
                        }
                    }),
                "+"
            );
            sum
        };

        // Generate the match arm.
        let this_pat = variant.pat();
        let other_pat = other_variant.pat();
        quote! {
            (&#this_pat, &#other_pat) => {
                Ok(#sum)
            }
        }
    }));

    if input.variants().len() > 1 {
        match_body = quote! { #match_body, _ => Err(()), };
    }

    // XXX: I think we might need to add more bounds? The original code added a
    // bound for the type of each field.
    input.bound_impl("::values::distance::ComputeSquaredDistance", quote! {
        #[allow(unused_variables, unused_imports)]
        #[inline]
        fn compute_squared_distance(
            &self,
            other: &Self,
        ) -> Result<::values::distance::SquaredDistance, ()> {
            match (self, other) {
                #match_body
            }
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
                impl<T> ::values::distance::ComputeSquaredDistance for A<T>
                where
                    T: ::values::distance::ComputeSquaredDistance
                {
                    #[allow(unused_variables, unused_imports)]
                    #[inline]
                    fn compute_squared_distance(
                        &self,
                        other: &Self,
                    ) -> Result<::values::distance::SquaredDistance, ()> {
                        match (self, other) {
                            (
                                &A { a: ref __binding_0, b: ref __binding_1, },
                                &A { a: ref other_0, b: ref other_1, }
                            ) => {
                                Ok(
                                    ::values::distance::ComputeSquaredDistance::compute_squared_distance(
                                        __binding_0,
                                        other_0,
                                    )? + ::values::distance::ComputeSquaredDistance::compute_squared_distance(
                                        __binding_1,
                                        other_1,
                                    )?
                                )
                            }
                        }
                    }
                }
            } no_build
        }
    }

    #[test]
    fn empty_variant() {
        test_derive! {
            super::derive {
                struct A;
            }
            expands to {
                impl ::values::distance::ComputeSquaredDistance for A {
                    #[allow(unused_variables, unused_imports)]
                    #[inline]
                    fn compute_squared_distance(
                        &self,
                        other: &Self,
                    ) -> Result<::values::distance::SquaredDistance, ()> {
                        match (self, other) {
                            (&A, &A) => {
                                Ok(::values::distance::SquaredDistance::Value(0.))
                            }
                        }
                    }
                }
            } no_build
        }
    }
}

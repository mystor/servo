/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote;
use synstructure;

pub fn derive(input: synstructure::Structure) -> quote::Tokens {
    let body = input.fold(false, |acc, bi| {
        quote!(#acc || ::style_traits::HasViewportPercentage::has_viewport_percentage(#bi))
    });
    input.bound_impl("::style_traits::HasViewportPercentage", quote! {
        #[allow(unused_variables, unused_imports)]
        #[inline]
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                #body
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
                impl<T> ::style_traits::HasViewportPercentage for A<T>
                where
                    T: ::style_traits::HasViewportPercentage
                {
                    #[allow(unused_variables, unused_imports)]
                    #[inline]
                    fn has_viewport_percentage(&self) -> bool {
                        match *self {
                            A {
                                a: ref __binding_0,
                                b: ref __binding_1,
                            } => {
                                false ||
                                    ::style_traits::HasViewportPercentage::has_viewport_percentage(__binding_0) ||
                                    ::style_traits::HasViewportPercentage::has_viewport_percentage(__binding_1)
                            }
                        }
                    }
                }
            } no_build
        }
    }
}


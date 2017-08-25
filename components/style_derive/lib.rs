/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;
#[macro_use] extern crate synstructure;

mod compute_squared_distance;
mod has_viewport_percentage;
mod to_animated_value;
mod to_computed_value;
mod to_css;

decl_derive!([ComputeSquaredDistance] => compute_squared_distance::derive);
decl_derive!([HasViewportPercentage] => has_viewport_percentage::derive);
decl_derive!([ToAnimatedValue] => to_animated_value::derive);
decl_derive!([ToComputedValue] => to_computed_value::derive);
decl_derive!([ToCss, attributes(css)] => to_css::derive);

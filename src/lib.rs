mod utils;
pub mod generic;
// import local package.
use crate::utils::gen_bigint_range;
// use external crates.
use rug::{rand::RandState, Complete, Integer};
use std::fmt;

use crate::generic::{MapResult, MappingError};
/// Source: Handbook of Applied Cryptography chapter-3
///         http://cacr.uwaterloo.ca/hac/about/chap3.pdf
/// rust programming by yangfh2004, January 2022

const BIG_INT_0: Integer = Integer::ZERO;

impl fmt::Display for MappingError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Error in mapping functions")
	}
}

fn func_f(x_i: &Integer, base: &Integer, y: &Integer, p: &Integer) -> MapResult<Integer> {
	match x_i.mod_u(3) {
		0 => Ok(Integer::from(x_i.pow_mod_ref(&Integer::from(2), p).unwrap())),
		1 => Ok(Integer::from(base * x_i).div_rem_euc_ref(p).complete().1),
		2 => Ok(Integer::from(y * x_i).div_rem_euc_ref(p).complete().1),
		_ => Err(MappingError),
	}
}

fn func_g(a: &Integer, n: &Integer, x_i: &Integer) -> MapResult<Integer> {
	match x_i.mod_u(3) {
		0 => Ok(Integer::from(a * 2).div_rem_euc_ref(n).complete().1),
		1 => Ok(Integer::from(a + 1).div_rem_euc_ref(n).complete().1),
		2 => Ok(a.clone()),
		_ => Err(MappingError),
	}
}

fn func_h(b: &Integer, n: &Integer, x_i: &Integer) -> MapResult<Integer> {
	match x_i.mod_u(3) {
		0 => Ok(Integer::from(b * 2).div_rem_euc_ref(n).complete().1),
		1 => Ok(b.clone()),
		2 => Ok(Integer::from(b + 1).div_rem_euc_ref(n).complete().1),
		_ => Err(MappingError),
	}
}

/// The equation to solve the private key from intermediate results of pollard rho algorithm.
/// If x_i == x_2i is True
///     ==> (base^(a1))*(y^(b1)) = (base^(a2))*(y^(b2)) (mod p)
///     ==> y^(b1 - b2) = base^(a2 - a1)                (mod p)
///     ==> base^((b1 - b2)*x) = base^(a2 - a1)         (mod p)
///     ==> (b1 - b2)*x = (a2 - a1)                     (mod n)
///     r = (b1 - b2) mod_floor (n)
///     if GCD(r, n) == 1 then,
///     ==> x = (r^(-1))*(a2 - a1)                      (mod n)
/// If `n` is not a prime number this algorithm will not be able to
/// solve the DLP, because GCD(r, n) != 1 then and one will have to
/// write an implementation to solve the equation:
///     (b1 - b2)*x = (a2 - a1) (mod n)
/// This equation will have multiple solutions out of which only one
/// will be the actual solution

pub fn eqs_solvers(
	a1: &Integer,
	b1: &Integer,
	a2: &Integer,
	b2: &Integer,
	n: &Integer,
) -> Option<Integer> {
	let r = Integer::from(b1 - b2).div_rem_euc_ref(n).complete().1;
	if r == 0 {
		None
	} else {
		match r.invert_ref(n) {
			Some(inv) => {
				let res_inv = Integer::from(inv);
				let dif = Integer::from(a2 - a1);
				Some(Integer::from(res_inv * dif).div_rem_euc_ref(n).complete().1)
			},
			None => {
				let div = r.gcd(n);
				// div is the first value of (g, x, y) as a result of gcd of r and n.
				let res_l = Integer::from(b1 - b2) / &div;
				let res_r = Integer::from(a2 - a2) / &div;
				let p1 = Integer::from(n / &div);
				match res_l.invert(&p1) {
					Ok(res_inv) =>
						Some(Integer::from(res_inv * res_r).div_rem_euc_ref(&p1).complete().1),
					Err(_) => None,
				}
			},
		}
	}
}

/// Refer to section 3.6.3 of Handbook of Applied Cryptography
/// Computes `x` = a mod n for the DLP base**x mod p == y
/// in the Group G = {0, 1, 2, ..., n}
/// given that order `n` is a prime number.
/// Since the RNG may not be thread-safe, it would be better to generate a RNG for each instance,
/// which has only small impact on overall performance.
/// # Arguments
/// * `seed` - An big integer as mersenne twister pseudorandom generator seed.
/// * `base` - Generator of the group.
/// * `y` - Result of base**x mod p.
/// * `p` - Group over which DLP is generated.
/// * `n` - Order of the group generated by `base`. Should be prime for this implementation.
pub fn pollard_rho(
	seed: &Integer,
	base: &Integer,
	y: &Integer,
	p: &Integer,
	n: &Integer,
) -> Option<Integer> {
	// Use mersenne twister algorithm to generate random numbers
	let mut rand = RandState::new_mersenne_twister();
	rand.seed(seed);
	let mut a_i: Integer = gen_bigint_range(&mut rand, &BIG_INT_0, n);
	let mut b_i: Integer = gen_bigint_range(&mut rand, &BIG_INT_0, n);
	let mut a_2i = a_i.clone();
	let mut b_2i = b_i.clone();
	let x_i_base = Integer::from(base.pow_mod_ref(&a_i, &p)?);
	let x_i_y = Integer::from(y.pow_mod_ref(&b_i, &p)?);
	let mut x_i = Integer::from(x_i_base * x_i_y).div_rem_euc_ref(p).complete().1;
	let mut x_2i = x_i.clone();
	let mut i = BIG_INT_0.clone();
	let mut xm_2i: Integer;
	let mut am_2i: Integer;
	let mut bm_2i: Integer;
	while &i < n {
		// Single Step calculations.
		a_i = func_g(&a_i, n, &x_i).expect("Mapping function g has error!");
		b_i = func_h(&b_i, n, &x_i).expect("Mapping function h has error!");
		x_i = func_f(&x_i, base, y, p).expect("Mapping function f has error!");
		// Double Step calculations
		xm_2i = func_f(&x_2i, base, y, p)
			.expect("Mapping function f has error in the intermediate step!");
		am_2i = func_g(&a_2i, n, &x_2i)
			.expect("Mapping function g has error in the intermediate step!");
		a_2i = func_g(&am_2i, n, &xm_2i).expect("Mapping function g has error in the final step!");
		bm_2i = func_h(&b_2i, n, &x_2i)
			.expect("Mapping function h has error in the intermediate step!");
		b_2i = func_h(&bm_2i, n, &xm_2i).expect("Mapping function h has error in the final step!");
		x_2i = func_f(&xm_2i, base, y, p).expect("Mapping function f has error in the final step!");
		if x_i == x_2i {
			return eqs_solvers(&a_i, &b_i, &a_2i, &b_2i, n)
		} else {
			i += 1;
		}
	}
	None
}

/// try to use pollard rho algorithm solve DLP problem with limited number of iterations.
pub fn try_pollard_rho(
	limit: usize,
	seed: &Integer,
	base: &Integer,
	y: &Integer,
	p: &Integer,
	n: &Integer,
) -> Integer {
	let mut loop_count = 0;
	let mut current_seed = seed.clone();
	loop {
		if let Some(key) = pollard_rho(&current_seed, &base, &y, &p, &n) {
			break key
		} else if loop_count < limit {
			// if cannot find solution with current seed, mutate the seed and try again.
			current_seed += 1;
			loop_count += 1;
		} else {
			// if cannot find the key after all trials, return zero.
			break Integer::ZERO
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_big_int_modulo_operator() {
		let num = Integer::from(-21);
		let four = Integer::from(4);
		let three = Integer::from(3);
		assert_eq!(
			num.div_rem_euc(four).1,
			three,
			"The remainder of euclidean division does not match!"
		);
	}

	#[test]
	fn test_pollard_rho() {
		let p = Integer::from(383);
		let n = Integer::from(191);
		let two = Integer::from(2);
		for i in 0..100 {
			// let num = gen_bigint_range(&mut rand, &two, &n);
			let num = Integer::from(57);
			let res = two.pow_mod_ref(&num, &p).unwrap();
			let y = Integer::from(res);
			let big_i = Integer::from(i);
			let key = try_pollard_rho(10, &big_i, &two, &y, &p, &n);
			let res_key = Integer::from(&num.div_rem_euc_ref(&n).complete().1);
			assert_eq!(&res_key, &key, "The found key {} is not the original key {}", key, num);
		}
	}
}

use rug::{rand::RandState, Integer};

/// These real versions are due to Kaisuki, 2021/01/07 added
/// modified by yangfh2004, 2022/01/31

pub fn gen_bigint_range(rand: &mut RandState, start: &Integer, stop: &Integer) -> Integer {
	let range = Integer::from(stop - start);
	let below = range.random_below(rand);
	start + below
}

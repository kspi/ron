use std::rand::random;

pub fn random_bernoulli(p: f64) -> bool {
    let x: f64 = random();
    x < p
}


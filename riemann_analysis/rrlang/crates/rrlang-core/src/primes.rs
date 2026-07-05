pub fn prime_sieve(max_index_inclusive: usize) -> Vec<bool> {
    if max_index_inclusive == 0 {
        return vec![false];
    }
    let mut is_prime = vec![true; max_index_inclusive + 1];
    is_prime[0] = false;
    if max_index_inclusive >= 1 {
        is_prime[1] = false;
    }
    let mut p = 2usize;
    while p * p <= max_index_inclusive {
        if is_prime[p] {
            let mut multiple = p * p;
            while multiple <= max_index_inclusive {
                is_prime[multiple] = false;
                multiple += p;
            }
        }
        p += 1;
    }
    is_prime
}

pub fn is_prime_number(value: usize) -> bool {
    if value < 2 {
        return false;
    }
    if value == 2 {
        return true;
    }
    if value % 2 == 0 {
        return false;
    }
    let mut candidate = 3usize;
    while candidate * candidate <= value {
        if value % candidate == 0 {
            return false;
        }
        candidate += 2;
    }
    true
}

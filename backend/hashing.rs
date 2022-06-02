use const_fnv1a_hash::fnv1a_hash_64;

pub const fn string_hash(input: &str) -> u64 {
    fnv1a_hash_64(input.as_bytes(), None)
}

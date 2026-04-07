use argon2::{Argon2, Params};

pub fn strong_argon2() -> Argon2<'static> {
    let params = Params::new(128 * 1024, 3, 1, None).unwrap();

    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
}

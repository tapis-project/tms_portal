use rand::CryptoRng;
use rand::rngs::ThreadRng;

pub fn generate_nonce() -> u32 {
    generate_nonce_ensure_crypto_rng(&mut ThreadRng::default())
}
fn generate_nonce_ensure_crypto_rng<T>(rng: &mut T) -> u32 where T:CryptoRng {
    rng.next_u32()
}

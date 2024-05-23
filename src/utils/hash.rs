use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn generate() -> String {
    let mut rng = rand::thread_rng();
    (0..30)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

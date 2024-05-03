pub struct WasmRng;

impl WasmRng {
    pub fn generate_key(length: usize) -> String {
        let mut rng = WasmRng {};
        let ascii = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

        (0..length)
            .map(|_| {
                let idx = rand_core::RngCore::next_u64(&mut rng) as usize % ascii.len();
                ascii.chars().nth(idx).unwrap()
            })
            .collect()
    }
}

impl rand_core::RngCore for WasmRng {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        if dest.len() < 8 {
            fill_8_bytes(&mut dest[..8].try_into().unwrap());
        } else {
            let window = dest.windows(8);

            for w in window {
                fill_8_bytes(&mut w.try_into().unwrap());
            }
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let mut w = [0u8; 8];
        fill_8_bytes(&mut w);
        u64::from_be_bytes(w)
    }
}

fn fill_8_bytes(w: &mut [u8; 8]) {
    let val = ft_sys::env::random().to_be_bytes();

    for (i, byte) in w.iter_mut().enumerate() {
        *byte = val[i];
    }
}

impl rand_core::CryptoRng for WasmRng {}

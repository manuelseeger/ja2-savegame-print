use super::header::SaveHeader;

mod table {
    include!("encryption_table.rs");
}

const ROTATION_SIZE: usize = 49;

/// English-edition `CalcJA2EncryptionSet` from Stracciatella SaveLoadGame.cc.
/// Arithmetic deliberately wraps exactly like upstream `UINT32` operations.
pub fn calculate_set(header: &SaveHeader) -> usize {
    let mut set = header.current_balance as u32;
    set = set.wrapping_mul(u32::from(header.mercs_on_team) + 1);
    set = set.wrapping_add_signed(i32::from(header.sector.z) * 3);
    set = set.wrapping_add(u32::from(header.load_screen_id));
    if header.alternate_sector {
        set = set.wrapping_add(7);
    }
    if header.random.is_multiple_of(2) {
        set = set.wrapping_add(1);
        if header.random.is_multiple_of(7) {
            set = set.wrapping_add(1);
            if header.random.is_multiple_of(23) {
                set = set.wrapping_add(1);
            }
            if header.random.is_multiple_of(79) {
                set = set.wrapping_add(2);
            }
        }
    }

    // The German-edition multiplier is intentionally unsupported.
    set %= 10;
    set = set.wrapping_add(header.day / 10);
    set %= 19;
    if header.gun_nut {
        set += 19 * 6;
    }
    if header.sci_fi {
        set += 19 * 3;
    }
    match header.difficulty {
        1 => {}
        2 => set += 19,
        3 => set += 19 * 2,
        _ => unreachable!("header validation accepts difficulty 1..=3"),
    }
    set as usize
}

/// Decode one independently encoded block using `NewJA2EncryptedFileRead`.
pub fn decrypt(encoded: &[u8], set: usize) -> Vec<u8> {
    debug_assert!(set < table::ENCRYPTION_ARRAY.len());
    let rotation = &table::ENCRYPTION_ARRAY[set];
    let mut previous = 0u8;
    encoded
        .iter()
        .enumerate()
        .map(|(index, &current)| {
            let plain = current
                .wrapping_sub(previous)
                .wrapping_sub(rotation[index % ROTATION_SIZE]);
            previous = current;
            plain
        })
        .collect()
}

#[cfg(test)]
fn encrypt(plain: &[u8], set: usize) -> Vec<u8> {
    let rotation = &table::ENCRYPTION_ARRAY[set];
    let mut previous = 0u8;
    plain
        .iter()
        .enumerate()
        .map(|(index, &current)| {
            let encoded = current
                .wrapping_add(previous)
                .wrapping_add(rotation[index % ROTATION_SIZE]);
            previous = encoded;
            encoded
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{decrypt, encrypt};

    #[test]
    fn decrypt_reverses_encrypt_for_every_rotation_set() {
        let plain: Vec<u8> = (0..=255).cycle().take(716).collect();

        for set in 0..228 {
            assert_eq!(decrypt(&encrypt(&plain, set), set), plain, "set {set}");
        }
    }

    #[test]
    fn decrypt_restarts_state_for_each_call() {
        let first = vec![1, 2, 3, 4];
        let second = vec![5, 6, 7, 8];

        assert_eq!(decrypt(&encrypt(&first, 42), 42), first);
        assert_eq!(decrypt(&encrypt(&second, 42), 42), second);
    }

    #[test]
    fn decrypt_matches_pinned_source_row_zero_vector() {
        // First rotation bytes are 11, 129, 18. Upstream's cumulative writer
        // encodes three zero plaintext bytes as 11, 140, 158.
        assert_eq!(decrypt(&[11, 140, 158], 0), [0, 0, 0]);
    }
}

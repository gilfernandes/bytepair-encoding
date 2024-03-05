use std::collections::HashMap;
use linked_hash_map::LinkedHashMap;

pub fn convert_to_bytes(input: &str) -> Vec<u16> {
    input.as_bytes().to_vec().iter().map(|x| *x as u16).collect()
}

fn get_stats(ids: Vec<u16>) -> HashMap<(u16, u16), u16> {
    let mut map = HashMap::new();
    for (n1, n2) in ids.iter().zip(ids.iter().skip(1)) {
        let count = map.entry((*n1, *n2)).or_insert(0);
        *count += 1;
    }
    map
}

/// Finds the most frequent consecutive pair of elements in a sequence of u8.
///
/// # Arguments
///
/// * `ids` - A vector of u8 elements to search through.
///
/// # Returns
///
/// An `Option` containing the most frequent pair of elements as `(u8, u8)`.
/// Returns `None` if the input vector is empty or has only one element.
pub fn get_most_frequent_pair(ids: Vec<u16>) -> Option<(u16, u16)> {
    let mut map = get_stats(ids);
    let option = map.iter().max_by_key(|&(_, count)| count);
    match option {
        None => return None,
        pair_count => {
            let (pair, count) = pair_count.unwrap();
            if *count < 2 {
                return None;
            }
            return Some(*pair);
        }
    }
}

pub fn merge(ids: Vec<u16>, pair: (u16, u16), idx: u16) -> Vec<u16> {
    if ids.len() < 2 {
        return ids;
    }
    let mut result = Vec::new();
    let mut skip_next = false;
    for (n1, n2) in ids.iter().zip(ids.iter().skip(1)) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if (*n1, *n2) == pair {
            result.push(idx);
            skip_next = true;
        } else {
            result.push(*n1);
        }
    }
    if !skip_next {
        result.push(*ids.last().unwrap());
    }
    result
}

fn calculate_merges_default(ids: Vec<u16>, vocab_size: u16) -> LinkedHashMap<(u16, u16), u16> {
    return calculate_merges(ids, vocab_size, 256);
}

fn calculate_merges(ids_orig: Vec<u16>, vocab_size: u16, vocab_start: u16) -> LinkedHashMap<(u16, u16), u16> {
    let num_merges = vocab_size - vocab_start;
    let mut ids = ids_orig.clone();
    let mut idx: u16;
    let mut merges = LinkedHashMap::new();
    for i in 0..num_merges {
        let pair = get_most_frequent_pair(ids.clone());
        if pair.is_none() {
            break;
        }
        idx = 256 + i;
        println!("merging {:?} into a new token {:?}", pair.unwrap(), idx);
        ids = merge(ids, pair.unwrap(), idx);
        merges.insert(pair.unwrap(), idx);
    }
    merges
}

fn generate_vocab(merges: LinkedHashMap<(u16, u16), u16>) -> LinkedHashMap<u16, Vec<u8>> {
    let mut vocab: LinkedHashMap<u16, Vec<u8>> = (0..256).map(|idx| (idx, vec![idx as u8])).collect();
    for ((p0, p1), idx) in merges.iter() {
        let val0 = vocab.get(p0).expect("p0 not found");
        let val1 = vocab.get(p1).expect("p0 not found");
        vocab.insert(*idx, val0.iter().chain(val1.iter()).cloned().collect());
    }
    vocab
}

fn generate_and_decode(ids_orig: Vec<u16>, merges: LinkedHashMap<(u16, u16), u16>) -> String {
    let vocab = generate_vocab(merges);
    decode(ids_orig, vocab)
}

fn decode(ids: Vec<u16>, vocab: LinkedHashMap<u16, Vec<u8>>) -> String {
    let mut res: Vec<u8> = Vec::new();
    for idx in ids.iter() {
        let value = vocab.get(idx).expect("idx not found");
        res.extend(value.iter());
    }
    String::from_utf8_lossy(&res).to_string()
}

fn get_pair_with_lowest_value(stats: HashMap<(u16, u16), u16>, merges: &LinkedHashMap<(u16, u16), u16>) -> (u16, u16) {
    let mut min = std::u16::MAX;
    let mut min_pair = (0, 0);
    for (pair, count) in stats.iter() {
        let res = merges.get(pair);
        if res.is_some() {
            let code = *res.unwrap();
            if code < min {
                min = code;
                min_pair = *pair;
            }
        }
    }
    min_pair
}

fn encode(input: &str, merges: LinkedHashMap<(u16, u16), u16>) -> Vec<u16> {
    let mut tokens = convert_to_bytes(input);
    while tokens.len() > 1 {
        let stats = get_stats(tokens.clone());
        let pair = get_pair_with_lowest_value(stats, &merges);
        if !merges.contains_key(&pair) {
            break;
        }
        let idx = merges.get(&pair).unwrap();
        tokens = merge(tokens, pair, *idx);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    const LONG_INPUT: &str = "Ｕｎｉｃｏｄｅ! 🅤🅝🅘🅒🅞🅓🅔‽ 🇺‌🇳‌🇮‌🇨‌🇴‌🇩‌🇪! 😄 The very name strikes fear and awe into the hearts of programmers worldwide. We all know we ought to “support Unicode” in our software (whatever that means—like using wchar_t for all the strings, right?). But Unicode can be abstruse, and diving into the thousand-page Unicode Standard plus its dozens of supplementary annexes, reports, and notes can be more than a little intimidating. I don’t blame programmers for still finding the whole thing mysterious, even 30 years after Unicode’s inception.";

    #[test]
    fn convert_to_bytes_simple() {
        let input = "hello";
        let bytes = convert_to_bytes(input);
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn convert_to_bytes_simple1() {
        let input = "hello ";
        let bytes = convert_to_bytes(input);
        assert_eq!(bytes, vec![104, 101, 108, 108, 111, 32]);
    }

    #[test]
    fn convert_to_bytes_complex() {
        let bytes = convert_to_bytes(LONG_INPUT);
        assert_eq!(bytes, vec![239, 188, 181, 239, 189, 142, 239, 189, 137, 239, 189, 131, 239, 189, 143, 239, 189, 132, 239, 189, 133, 33, 32, 240, 159, 133, 164, 240, 159, 133, 157, 240, 159, 133, 152, 240, 159, 133, 146, 240, 159, 133, 158, 240, 159, 133, 147, 240, 159, 133, 148, 226, 128, 189, 32, 240, 159, 135, 186, 226, 128, 140, 240, 159, 135, 179, 226, 128, 140, 240, 159, 135, 174, 226, 128, 140, 240, 159, 135, 168, 226, 128, 140, 240, 159, 135, 180, 226, 128, 140, 240, 159, 135, 169, 226, 128, 140, 240, 159, 135, 170, 33, 32, 240, 159, 152, 132, 32, 84, 104, 101, 32, 118, 101, 114, 121, 32, 110, 97, 109, 101, 32, 115, 116, 114, 105, 107, 101, 115, 32, 102, 101, 97, 114, 32, 97, 110, 100, 32, 97, 119, 101, 32, 105, 110, 116, 111, 32, 116, 104, 101, 32, 104, 101, 97, 114, 116, 115, 32, 111, 102, 32, 112, 114, 111, 103, 114, 97, 109, 109, 101, 114, 115, 32, 119, 111, 114, 108, 100, 119, 105, 100, 101, 46, 32, 87, 101, 32, 97, 108, 108, 32, 107, 110, 111, 119, 32, 119, 101, 32, 111, 117, 103, 104, 116, 32, 116, 111, 32, 226, 128, 156, 115, 117, 112, 112, 111, 114, 116, 32, 85, 110, 105, 99, 111, 100, 101, 226, 128, 157, 32, 105, 110, 32, 111, 117, 114, 32, 115, 111, 102, 116, 119, 97, 114, 101, 32, 40, 119, 104, 97, 116, 101, 118, 101, 114, 32, 116, 104, 97, 116, 32, 109, 101, 97, 110, 115, 226, 128, 148, 108, 105, 107, 101, 32, 117, 115, 105, 110, 103, 32, 119, 99, 104, 97, 114, 95, 116, 32, 102, 111, 114, 32, 97, 108, 108, 32, 116, 104, 101, 32, 115, 116, 114, 105, 110, 103, 115, 44, 32, 114, 105, 103, 104, 116, 63, 41, 46, 32, 66, 117, 116, 32, 85, 110, 105, 99, 111, 100, 101, 32, 99, 97, 110, 32, 98, 101, 32, 97, 98, 115, 116, 114, 117, 115, 101, 44, 32, 97, 110, 100, 32, 100, 105, 118, 105, 110, 103, 32, 105, 110, 116, 111, 32, 116, 104, 101, 32, 116, 104, 111, 117, 115, 97, 110, 100, 45, 112, 97, 103, 101, 32, 85, 110, 105, 99, 111, 100, 101, 32, 83, 116, 97, 110, 100, 97, 114, 100, 32, 112, 108, 117, 115, 32, 105, 116, 115, 32, 100, 111, 122, 101, 110, 115, 32, 111, 102, 32, 115, 117, 112, 112, 108, 101, 109, 101, 110, 116, 97, 114, 121, 32, 97, 110, 110, 101, 120, 101, 115, 44, 32, 114, 101, 112, 111, 114, 116, 115, 44, 32, 97, 110, 100, 32, 110, 111, 116, 101, 115, 32, 99, 97, 110, 32, 98, 101, 32, 109, 111, 114, 101, 32, 116, 104, 97, 110, 32, 97, 32, 108, 105, 116, 116, 108, 101, 32, 105, 110, 116, 105, 109, 105, 100, 97, 116, 105, 110, 103, 46, 32, 73, 32, 100, 111, 110, 226, 128, 153, 116, 32, 98, 108, 97, 109, 101, 32, 112, 114, 111, 103, 114, 97, 109, 109, 101, 114, 115, 32, 102, 111, 114, 32, 115, 116, 105, 108, 108, 32, 102, 105, 110, 100, 105, 110, 103, 32, 116, 104, 101, 32, 119, 104, 111, 108, 101, 32, 116, 104, 105, 110, 103, 32, 109, 121, 115, 116, 101, 114, 105, 111, 117, 115, 44, 32, 101, 118, 101, 110, 32, 51, 48, 32, 121, 101, 97, 114, 115, 32, 97, 102, 116, 101, 114, 32, 85, 110, 105, 99, 111, 100, 101, 226, 128, 153, 115, 32, 105, 110, 99, 101, 112, 116, 105, 111, 110, 46]);
    }

    #[test]
    fn get_most_frequent_pair_complex() {
        let ids = convert_to_bytes(LONG_INPUT);
        let map = get_most_frequent_pair(ids);
        assert_eq!(map, Some((101u16, 32u16)), "The most frequent pair did not match the expected values.");
    }

    #[test]
    fn merge_complex() {
        let ids = convert_to_bytes(LONG_INPUT);
        let map = get_most_frequent_pair(ids.clone());
        assert!(map.is_some(), "The most frequent pair was not found.");
        let pair = map.unwrap();
        let idx = 256;
        let merged = merge(ids, pair, idx);
        println!("Merged: {:?}", merged);
        let mut count = 0;
        for i in merged.iter() {
            if idx == *i {
                count += 1;
            }
        }
        assert_eq!(count, 20, "The count of merges should be 20");
    }

    #[test]
    fn merge_continuous() {
        let ids = vec![101, 32, 101, 32, 101, 32, 101];
        let idx = 256;
        let merged = merge(ids, (101, 32), idx);
        assert_eq!(merged, vec![256, 256, 256, 101]);
        println!("Merged: {:?}", merged);
    }

    #[test]
    fn calculate_merges_simple() {
        let input = "aaabdaaabac";
        let ids = convert_to_bytes(input);
        let result = calculate_merges_default(ids, 276);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn calculate_merges_complex() {
        let merges = run_calculate_merges();
        assert_eq!(merges.len(), 20);
    }

    #[test]
    fn generate_vocab_complex() {
        let merges = run_calculate_merges();
        let vocab = generate_vocab(merges);
        println!("Vocab: {:?}", vocab);
    }

    #[test]
    fn decode_simple() {
        let merges = run_calculate_merges();
        let vocab = generate_vocab(merges);
        let res = decode(vec![101,32], vocab);
        println!("Decoded: {:?}", res);
    }

    #[test]
    fn encode_simple() {
        let merges = run_calculate_merges();
        let encoded = encode("hello world", merges);
        println!("Encoded: {:?}", encoded);
    }

    #[test]
    fn encode_decode_simple() {
        let merges = run_calculate_merges();
        let orig_str = "hello world, While I'm glad to hear about your job opportunity in Dubai, I must admit that I'm a bit skeptical about the salary you mentioned. Ｕｎｉｃｏｄｅ! 🅤🅝🅘🅒🅞🅓🅔‽";
        let encoded = encode(orig_str, merges.clone());
        println!("Encoded: {:?}", encoded);
        let vocab = generate_vocab(merges);
        let res = decode(encoded.clone(), vocab);
        println!("Decoded: {:?}", res);
        println!("Compression: {:?}", orig_str.len() as f64 / encoded.len() as f64);
        assert_eq!(res, orig_str);
    }

    fn run_calculate_merges() -> LinkedHashMap<(u16, u16), u16> {
        let input = read_complex_file();
        let ids = convert_to_bytes(&input);
        let merges: LinkedHashMap<(u16, u16), u16> = calculate_merges_default(ids, 276);
        merges
    }

    fn read_complex_file() -> String {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut path = PathBuf::from(manifest_dir);
        path.push("test");
        path.push("resources");
        path.push("unicode_input.txt");
        println!("unicode file path: {}", path.display());
        assert!(path.exists());
        let input = fs::read_to_string(path).expect("Unable to read file");
        input
    }
}

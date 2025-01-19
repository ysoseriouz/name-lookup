use bloom_filter_yss::bloom_filter::BloomFilter;

fn lookup(bloom_filter: &BloomFilter, s: &str) {
    if bloom_filter.lookup(s) {
        println!("Exist: {}", s);
    } else {
        println!("Not exist: {}", s);
    }
}

fn main() {
    let mut bloom_filter = BloomFilter::new(10);
    bloom_filter.insert("abound");
    bloom_filter.insert("abound1");
    bloom_filter.insert("abound");

    lookup(&bloom_filter, "abound");
    lookup(&bloom_filter, "aboundd");

    bloom_filter.insert("test");

    lookup(&bloom_filter, "test");
}

A Rust implementation of [xxHash](http://code.google.com/p/xxhash/).

    $ ./xxhash --test --bench

    running 13 tests
    test xxh64::test_chunks ... ok
    test xxh64::test_hash_idempotent ... ok
    test xxh64::test_hash_no_bytes_dropped_64 ... ok
    test xxh64::test_hash_no_bytes_dropped_32 ... ok
    test xxh64::test_hash_no_concat_alias ... ok
    test xxh64::test_hash_uint ... ok
    test xxh64::test_oneshot ... ok
    test xxh64::bench_64k_oneshot       ... bench:     16599 ns/iter (+/- 138) = 3948 MB/s
    test xxh64::bench_long_str          ... bench:       213 ns/iter (+/- 3) = 2093 MB/s
    test xxh64::bench_str_of_8_bytes    ... bench:        31 ns/iter (+/- 2) = 258 MB/s
    test xxh64::bench_str_over_8_bytes  ... bench:        79 ns/iter (+/- 0) = 126 MB/s
    test xxh64::bench_str_under_8_bytes ... bench:        55 ns/iter (+/- 3) = 54 MB/s
    test xxh64::bench_u64               ... bench:        48 ns/iter (+/- 3) = 166 MB/s

    test result: ok. 7 passed; 0 failed; 0 ignored; 6 measured




use std::thread;

use lazy_static::lazy_static;
use pretty_assertions::assert_eq;

use super::*;

lazy_static! {
    static ref TTL: Duration = Duration::minutes(1);
}

fn print_cache_list(header: &'static str) {
    println!("\n+--- Cache {} ---", header);
    wrapper::list_cache()
        .filter_map(|res| res.ok())
        .for_each(|meta| {
            let age_ms = meta.time;
            let cache_age = chrono::Utc.timestamp((age_ms / 1000) as i64, (age_ms % 1000) as u32);
            eprintln!(
                "| - {}\n|   SIZE: {}\n|   AGE: {}",
                meta.key, meta.size, cache_age
            )
        });
    println!("+{}", "-".repeat(header.len() + 14));
}

#[test]
fn test_cache_is_empty() {
    let read = try_load_cache("test cache entry", Duration::max_value()).unwrap();
    print_cache_list("Cache");
    assert_eq!(read, CacheResult::Miss);
}

#[test]
fn basic_caching() {
    let url = "http://invalid.local/test";
    // Cache is empty
    let val = try_load_cache(url, Duration::max_value()).unwrap();
    print_cache_list("After first read");
    assert_eq!(val, CacheResult::Miss);
    // Populate the cache with the first request
    let val = fetch(url, *TTL, |txt, _| Ok(txt)).unwrap();
    assert_eq!(val, "It works",);
    // The cache should now be hit
    let val = try_load_cache(url, Duration::max_value()).unwrap();
    print_cache_list("After second read");
    assert_eq!(
        val,
        CacheResult::Hit((
            "It works".into(),
            Headers {
                etag: Some("static".into()),
                this_page: Some(1),
                next_page: None,
                last_page: Some(1),
            }
        ))
    );
    // Let's fake a stale entry
    thread::sleep(std::time::Duration::from_secs(1));
    let val = try_load_cache(url, Duration::zero()).unwrap();
    assert!(matches!(val, CacheResult::Stale(_, _)));
}

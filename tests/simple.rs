use std::net::Ipv6Addr;

#[conformance::test(exact, serde=yaml, file="tests/simple.test")]
fn example(s: &str) -> Result<Ipv6Addr, String> {
    s.trim().parse::<Ipv6Addr>().map_err(|e| e.to_string())
}

use super::checks::check_port_available;

#[test]
fn check_port_available_passes_for_unused_high_port() {
    let result = check_port_available("127.0.0.1", 59999);
    assert!(result.ok, "unused high port should be available");
}

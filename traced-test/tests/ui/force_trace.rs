use traced_test::traced_test;

#[traced_test(trace = true)]
fn EXPECT_TRACE_DISPLAYED_test_always_trace() {
    info!("This log should always be displayed.");
    assert!(true);
}

-- Phase 199 common test fixtures.
--
-- Fixtures describe deterministic provider/profile inputs for tests. They are
-- pure data helpers and do not grant provider authority.

pub type DeterministicProviderProfileFixture = DeterministicProviderProfileFixture {
    profile: String,
    seed: Int,
    clock_epoch_millis: Int,
    http_host: String,
    filesystem_root: String,
    logging_enabled: Bool,
};

pub type CommonTestCase = CommonTestCase {
    name: String,
    input: String,
    expected: String,
};

pub fn deterministic_profile_fixture(profile: String, seed: Int) -> DeterministicProviderProfileFixture {
    DeterministicProviderProfileFixture {
        profile: profile,
        seed: seed,
        clock_epoch_millis: 0,
        http_host: "example.test",
        filesystem_root: "/tmp/ash-test",
        logging_enabled: true,
    }
}

pub fn test_clock_fixture(epoch_millis: Int) -> DeterministicProviderProfileFixture {
    DeterministicProviderProfileFixture {
        profile: "test-clock",
        seed: 0,
        clock_epoch_millis: epoch_millis,
        http_host: "example.test",
        filesystem_root: "/tmp/ash-test",
        logging_enabled: false,
    }
}

pub fn common_test_case(name: String, input: String, expected: String) -> CommonTestCase {
    CommonTestCase { name: name, input: input, expected: expected }
}

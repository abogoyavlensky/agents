"""Unit tests for the pure policy functions of the agent-proxy allowlist addon.

These run anywhere Python 3 is available; mitmproxy is not needed, because the
addon guards its mitmproxy imports.
"""

import unittest

from allowlist_addon import decide, parse_rules


class ParseRulesTest(unittest.TestCase):
    def test_comments_and_blanks_are_ignored(self):
        rules = parse_rules("# a comment\n\n  \ngithub.com\n  # indented comment\n")
        self.assertEqual(rules.domains, frozenset({"github.com"}))
        self.assertFalse(rules.allow_all)

    def test_entries_are_lowercased_and_stripped(self):
        rules = parse_rules("  GitHub.COM  \n\tApi.GitHub.com\n")
        self.assertEqual(rules.domains, frozenset({"github.com", "api.github.com"}))

    def test_leading_dot_is_stripped(self):
        rules = parse_rules(".github.com\n")
        self.assertEqual(rules.domains, frozenset({"github.com"}))

    def test_trailing_dot_is_stripped(self):
        rules = parse_rules("github.com.\n")
        self.assertEqual(rules.domains, frozenset({"github.com"}))

    def test_wildcard_prefix_is_treated_as_the_bare_domain(self):
        rules = parse_rules("*.github.com\n")
        self.assertEqual(rules.domains, frozenset({"github.com"}))
        self.assertFalse(rules.allow_all)

    def test_star_sets_allow_all(self):
        rules = parse_rules("# discovery mode\n*\n")
        self.assertTrue(rules.allow_all)

    def test_star_with_other_entries_still_sets_allow_all(self):
        rules = parse_rules("github.com\n*\n")
        self.assertTrue(rules.allow_all)
        self.assertEqual(rules.domains, frozenset({"github.com"}))

    def test_empty_text_gives_no_domains_and_no_allow_all(self):
        rules = parse_rules("")
        self.assertEqual(rules.domains, frozenset())
        self.assertFalse(rules.allow_all)

    def test_inline_comment_is_stripped(self):
        rules = parse_rules("github.com  # git remotes\n")
        self.assertEqual(rules.domains, frozenset({"github.com"}))


class DecideTest(unittest.TestCase):
    def setUp(self):
        self.rules = parse_rules("github.com\napi.anthropic.com\n")
        self.all = parse_rules("*\n")

    def assertDenied(self, result, reason):
        allowed, got = result
        self.assertFalse(allowed)
        self.assertEqual(got, reason)

    def test_exact_match_allowed(self):
        allowed, reason = decide("github.com", 443, "connect", self.rules)
        self.assertTrue(allowed)
        self.assertEqual(reason, "allowlist")

    def test_subdomain_allowed(self):
        allowed, _ = decide("api.github.com", 443, "connect", self.rules)
        self.assertTrue(allowed)

    def test_deep_subdomain_allowed(self):
        allowed, _ = decide("a.b.github.com", 443, "connect", self.rules)
        self.assertTrue(allowed)

    def test_uppercase_host_allowed(self):
        allowed, _ = decide("API.GitHub.com", 443, "connect", self.rules)
        self.assertTrue(allowed)

    def test_sibling_domain_denied(self):
        self.assertDenied(
            decide("notgithub.com", 443, "connect", self.rules), "not-in-allowlist"
        )

    def test_suffix_without_dot_boundary_denied(self):
        self.assertDenied(
            decide("evilgithub.com", 443, "connect", self.rules), "not-in-allowlist"
        )

    def test_allow_all_allows_unknown_host(self):
        allowed, reason = decide("example.com", 443, "connect", self.all)
        self.assertTrue(allowed)
        self.assertEqual(reason, "allow-all")

    def test_ipv4_literal_denied_even_with_allow_all(self):
        self.assertDenied(decide("1.1.1.1", 443, "connect", self.all), "ip-literal")

    def test_ipv6_literal_denied_even_with_allow_all(self):
        self.assertDenied(decide("::1", 443, "connect", self.all), "ip-literal")

    def test_bracketed_ipv6_literal_denied(self):
        self.assertDenied(decide("[::1]", 443, "connect", self.all), "ip-literal")

    def test_localhost_denied_even_with_allow_all(self):
        self.assertDenied(decide("localhost", 443, "connect", self.all), "local-name")

    def test_localhost_subdomain_denied(self):
        self.assertDenied(
            decide("foo.localhost", 443, "connect", self.all), "local-name"
        )

    def test_mdns_name_denied(self):
        self.assertDenied(decide("foo.local", 443, "connect", self.all), "local-name")

    def test_internal_name_denied(self):
        self.assertDenied(
            decide("foo.internal", 443, "connect", self.all), "local-name"
        )

    def test_lima_host_denied(self):
        self.assertDenied(
            decide("host.lima.internal", 443, "connect", self.all), "local-name"
        )

    def test_connect_on_other_port_denied(self):
        self.assertDenied(decide("github.com", 8443, "connect", self.all), "bad-port")

    def test_connect_on_port_80_denied(self):
        self.assertDenied(decide("github.com", 80, "connect", self.all), "bad-port")

    def test_http_on_port_80_allowed(self):
        allowed, _ = decide("github.com", 80, "http", self.rules)
        self.assertTrue(allowed)

    def test_http_on_other_port_denied(self):
        self.assertDenied(decide("github.com", 8080, "http", self.all), "bad-port")

    def test_http_on_port_443_denied(self):
        self.assertDenied(decide("github.com", 443, "http", self.all), "bad-port")

    def test_no_rules_denies_everything(self):
        self.assertDenied(decide("github.com", 443, "connect", None), "no-allowlist")

    def test_empty_allowlist_denies(self):
        self.assertDenied(
            decide("github.com", 443, "connect", parse_rules("")), "not-in-allowlist"
        )

    def test_missing_host_denied(self):
        self.assertDenied(decide("", 443, "connect", self.all), "no-host")

    def test_unknown_kind_denied(self):
        self.assertDenied(decide("github.com", 443, "gopher", self.all), "bad-kind")

    def test_denial_reasons_are_distinct(self):
        reasons = {
            decide("github.com", 443, "connect", None)[1],
            decide("", 443, "connect", self.all)[1],
            decide("1.1.1.1", 443, "connect", self.all)[1],
            decide("localhost", 443, "connect", self.all)[1],
            decide("github.com", 8443, "connect", self.all)[1],
            decide("github.com", 443, "gopher", self.all)[1],
            decide("notgithub.com", 443, "connect", self.rules)[1],
        }
        self.assertEqual(len(reasons), 7)


if __name__ == "__main__":
    unittest.main()

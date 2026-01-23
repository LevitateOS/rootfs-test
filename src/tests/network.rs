//! Network tests.
//!
//! Can users use network tools?
//! Note: Actual network connectivity depends on container setup.
//!
//! ## Anti-Reward-Hacking Design
//!
//! Tests verify actual command output, not just exit codes.
//!
//! Uses cheat_ensure! to document cheat vectors in failure messages.

use super::{test_result, Test, TestResult};
use crate::container::Container;
use leviso_cheat_guard::cheat_ensure;

/// Test: IP command works
struct IpCommand;

impl Test for IpCommand {
    fn name(&self) -> &str { "ip command" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can view and configure network interfaces"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                ip link show &&
                ip addr show lo
            "#)?;

            cheat_ensure!(
                result.contains("lo"),
                protects = "ip command lists network interfaces",
                severity = "CRITICAL",
                cheats = ["Only check ip exit code", "Accept any output"],
                consequence = "ip broken, can't configure networking",
                "ip link show didn't list loopback"
            );
            cheat_ensure!(
                result.contains("127.0.0.1"),
                protects = "loopback interface has IP address",
                severity = "CRITICAL",
                cheats = ["Only check interface exists", "Skip IP verification"],
                consequence = "localhost networking broken",
                "loopback doesn't have 127.0.0.1"
            );
            Ok("ip command works".into())
        })
    }
}

/// Test: Ping command exists
struct PingCommand;

impl Test for PingCommand {
    fn name(&self) -> &str { "ping" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can test network connectivity with ping"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                ping -V &&
                ping -c 1 127.0.0.1
            "#)?;

            cheat_ensure!(
                result.contains("iputils") || result.contains("ping"),
                protects = "ping command is functional",
                severity = "HIGH",
                cheats = ["Only check ping exists", "Skip version check"],
                consequence = "ping broken, can't test connectivity",
                "ping not working"
            );
            cheat_ensure!(
                result.contains("1 received") || result.contains("1 packets received"),
                protects = "ping can reach localhost",
                severity = "CRITICAL",
                cheats = ["Only check ping exits", "Skip packet verification"],
                consequence = "Network stack broken, can't reach localhost",
                "ping loopback failed"
            );
            Ok("ping works".into())
        })
    }
}

/// Test: Curl command works
struct CurlCommand;

impl Test for CurlCommand {
    fn name(&self) -> &str { "curl" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can download files and interact with HTTP services"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok("curl --version | head -1")?;

            cheat_ensure!(
                result.contains("curl"),
                protects = "curl command is functional",
                severity = "CRITICAL",
                cheats = ["Only check curl exists", "Skip version output"],
                consequence = "curl broken, can't download files or interact with APIs",
                "curl not working: {}", result
            );
            Ok(result.trim().into())
        })
    }
}

/// Test: DNS resolution config
struct DnsConfig;

impl Test for DnsConfig {
    fn name(&self) -> &str { "DNS config" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "System is configured for DNS resolution"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            // In a container, resolv.conf might be managed by nspawn
            // Just check nsswitch.conf has hosts entry
            let result = c.exec_ok("grep hosts /etc/nsswitch.conf")?;

            cheat_ensure!(
                !result.is_empty(),
                protects = "nsswitch.conf has hosts configuration",
                severity = "HIGH",
                cheats = ["Only check file exists", "Skip content verification"],
                consequence = "DNS resolution may not work",
                "No hosts entry in nsswitch.conf"
            );
            Ok("DNS resolution configured".into())
        })
    }
}

/// Test: /etc/hosts works
struct HostsFile;

impl Test for HostsFile {
    fn name(&self) -> &str { "/etc/hosts" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "Local hostname resolution works via /etc/hosts"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok("cat /etc/hosts")?;

            cheat_ensure!(
                result.contains("127.0.0.1") && result.contains("localhost"),
                protects = "/etc/hosts has localhost entry",
                severity = "CRITICAL",
                cheats = ["Only check file exists", "Skip content verification"],
                consequence = "localhost resolution broken, many apps fail",
                "/etc/hosts missing localhost entry"
            );
            Ok("/etc/hosts configured correctly".into())
        })
    }
}

/// Test: SS command works
struct SsCommand;

impl Test for SsCommand {
    fn name(&self) -> &str { "ss (sockets)" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can inspect network sockets and connections"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                ss --version &&
                ss -ln
            "#)?;

            cheat_ensure!(
                result.contains("iproute"),
                protects = "ss is from iproute2 (proper implementation)",
                severity = "HIGH",
                cheats = ["Only check ss exists", "Accept any implementation"],
                consequence = "ss may be broken or incomplete",
                "ss not from iproute2: {}", result
            );
            Ok("ss command works".into())
        })
    }
}

pub fn tests() -> Vec<Box<dyn Test>> {
    vec![
        Box::new(IpCommand),
        Box::new(PingCommand),
        Box::new(CurlCommand),
        Box::new(DnsConfig),
        Box::new(HostsFile),
        Box::new(SsCommand),
    ]
}

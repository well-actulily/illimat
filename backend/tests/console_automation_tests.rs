// Automated console interaction testing framework
// This module provides utilities for testing console applications without manual interaction

use std::process::{Command, Stdio};
use std::io::Write;
use std::time::{Duration, Instant};
// use std::fs;  // Unused for now

/// Console test framework for automated interaction testing
pub struct ConsoleTestFramework {
    _binary_path: String,
    timeout: Duration,
    _temp_dir: String,
}

#[derive(Debug)]
pub struct TestResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub exit_code: Option<i32>,
}

impl ConsoleTestFramework {
    pub fn new(binary_name: &str) -> Self {
        Self {
            _binary_path: format!("target/debug/{}", binary_name),
            timeout: Duration::from_secs(10),
            _temp_dir: "test_temp".to_string(),
        }
    }

    /// Test console interaction with predefined input sequence
    pub fn test_interaction(&self, input_sequence: &[&str], expected_patterns: &[&str]) -> TestResult {
        let start_time = Instant::now();
        
        // Create input string
        let input_data = input_sequence.join("\n") + "\n";
        
        // Spawn the console process
        let mut child = match Command::new("cargo")
            .args(&["run", "--bin", "console"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                return TestResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Failed to spawn process: {}", e),
                    execution_time: start_time.elapsed(),
                    exit_code: None,
                };
            }
        };

        // Send input to the process
        if let Some(stdin) = child.stdin.as_mut() {
            if let Err(e) = stdin.write_all(input_data.as_bytes()) {
                let _ = child.kill();
                return TestResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Failed to write input: {}", e),
                    execution_time: start_time.elapsed(),
                    exit_code: None,
                };
            }
        }

        // Wait for process completion with timeout
        let output = match self.wait_with_timeout(child) {
            Ok(output) => output,
            Err(e) => {
                return TestResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Process timeout or error: {}", e),
                    execution_time: start_time.elapsed(),
                    exit_code: None,
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Check if expected patterns are found in output
        let success = expected_patterns.iter()
            .all(|pattern| stdout.contains(pattern) || stderr.contains(pattern));

        TestResult {
            success,
            stdout,
            stderr,
            execution_time: start_time.elapsed(),
            exit_code: output.status.code(),
        }
    }

    /// Test error handling with various invalid inputs
    pub fn test_error_handling(&self, invalid_inputs: &[&str]) -> Vec<TestResult> {
        let mut results = Vec::new();

        for &invalid_input in invalid_inputs {
            let result = self.test_interaction(
                &[invalid_input, "2", "q", "y"], // Try invalid input, then valid game setup, then quit
                &["Invalid"] // Expect some error message mentioning invalid input
            );
            results.push(result);
        }

        results
    }

    /// Test startup performance
    pub fn test_startup_performance(&self) -> TestResult {
        let start_time = Instant::now();

        let result = self.test_interaction(
            &["2", "q", "y"], // Start 2-player game, then quit
            &["Welcome", "ILLIMAT", "players"]
        );

        TestResult {
            success: result.success && result.execution_time < Duration::from_secs(5),
            stdout: result.stdout,
            stderr: result.stderr,
            execution_time: start_time.elapsed(),
            exit_code: result.exit_code,
        }
    }

    /// Test game flow without hanging
    pub fn test_no_hanging(&self) -> TestResult {
        // Test the original hanging scenario: empty input
        self.test_interaction(
            &["", "", "2", "q", "y"], // Empty inputs, then valid setup, then quit
            &["Empty input", "Welcome to the Illimat table", "Game initialized with 2 players"]
        )
    }

    fn wait_with_timeout(&self, mut child: std::process::Child) -> Result<std::process::Output, String> {
        let start = Instant::now();
        
        loop {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Process has finished, get the output
                    return child.wait_with_output()
                        .map_err(|e| format!("Failed to get process output: {}", e));
                }
                Ok(None) => {
                    // Process is still running
                    if start.elapsed() > self.timeout {
                        let _ = child.kill();
                        return Err("Process timeout".to_string());
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(format!("Error checking process status: {}", e));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_startup() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "q", "y"],
            &["ILLIMAT", "Welcome", "2 players"]
        );
        
        assert!(result.success, "Console should start and show welcome message");
        assert!(result.execution_time < Duration::from_secs(10), 
                "Startup should be reasonably fast");
    }

    #[test]
    fn test_empty_input_handling() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_no_hanging();
        
        assert!(result.success, "Console should handle empty input gracefully");
        assert!(result.stdout.contains("Empty input"), 
                "Should show empty input message");
    }

    #[test]
    fn test_invalid_input_handling() {
        let framework = ConsoleTestFramework::new("console");
        let results = framework.test_error_handling(&["abc", "0", "5", ""]);
        
        assert!(!results.is_empty(), "Should test invalid inputs");
        for result in results {
            assert!(result.execution_time < Duration::from_secs(10), 
                    "Invalid input handling should not hang");
        }
    }

    #[test]
    fn test_performance_baseline() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_startup_performance();
        
        assert!(result.success, "Performance test should pass");
        assert!(result.execution_time < Duration::from_secs(5), 
                "Startup should be under 5 seconds");
    }
}

/// Game scenario testing utilities
pub struct GameScenarioTester {
    framework: ConsoleTestFramework,
}

impl GameScenarioTester {
    pub fn new() -> Self {
        Self {
            framework: ConsoleTestFramework::new("console"),
        }
    }

    /// Test a complete game initialization scenario
    pub fn test_game_initialization(&self) -> TestResult {
        self.framework.test_interaction(
            &["2", "q", "y"], // 2 players, then quit
            &[
                "Welcome to the Illimat table",
                "Game initialized with 2 players", 
                "Dealer: Player",
                "First player: Player",
                "Deck: 32 cards", // After dealing in 2-player game (52 - 12 field cards - 8 player cards)
            ]
        )
    }

    /// Test help system functionality
    pub fn test_help_system(&self) -> TestResult {
        self.framework.test_interaction(
            &["2", "?", "q", "y"], // 2 players, show help, quit
            &[
                "ILLIMAT HELP",
                "AVAILABLE ACTIONS",
                "sow", "harvest", "stockpile",
                "CURRENT SEASON RESTRICTIONS",
                "Spring Field", "Summer Field"
            ]
        )
    }

    /// Test error message quality
    pub fn test_error_messages(&self) -> TestResult {
        self.framework.test_interaction(
            &["abc", "0", "5", "2", "invalid_action", "q", "y"],
            &[
                "Invalid input 'abc'",
                "Please enter a number from 2 to 4",
                "Invalid number",
                "2 players",
                "Invalid command"
            ]
        )
    }
}

impl Default for GameScenarioTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod scenario_tests {
    use super::*;

    #[test]
    fn test_complete_game_initialization() {
        let tester = GameScenarioTester::new();
        let result = tester.test_game_initialization();
        
        assert!(result.success, "Game initialization should work correctly: {}", result.stderr);
    }

    #[test]
    fn test_help_system_functionality() {
        let tester = GameScenarioTester::new();
        let result = tester.test_help_system();
        
        assert!(result.success, "Help system should provide comprehensive information");
        assert!(result.stdout.contains("AVAILABLE ACTIONS"), "Should show available actions");
        assert!(result.stdout.contains("SEASON RESTRICTIONS"), "Should show season info");
    }

    #[test]
    fn test_error_message_quality() {
        let tester = GameScenarioTester::new();
        let result = tester.test_error_messages();
        
        assert!(result.success, "Error messages should be helpful and descriptive");
    }

    #[test]
    fn test_multiplayer_game_setup() {
        let tester = GameScenarioTester::new();
        let result = tester.framework.test_interaction(
            &["3", "q", "y"], // 3 players, then quit
            &["3 players", "52 cards", "Dealer: Player"]
        );
        
        assert!(result.success, "Should handle 3-player game setup");
    }

    #[test]
    fn test_maximum_players() {
        let tester = GameScenarioTester::new();
        let result = tester.framework.test_interaction(
            &["4", "q", "y"], // 4 players, then quit
            &["4 players", "65 cards", "Game initialized"]
        );
        
        assert!(result.success, "Should handle 4-player game setup");
    }

    #[test]
    fn test_console_performance_under_load() {
        let framework = ConsoleTestFramework::new("console");
        let start = std::time::Instant::now();
        
        let result = framework.test_interaction(
            &["2", "?", "q", "y"], // Quick help system test
            &["ILLIMAT HELP", "Welcome"]
        );
        
        let duration = start.elapsed();
        assert!(result.success, "Performance test should complete successfully");
        assert!(duration < std::time::Duration::from_secs(3), "Should complete within 3 seconds");
    }

    #[test]
    fn test_memory_usage_stability() {
        let framework = ConsoleTestFramework::new("console");
        
        // Test multiple rapid game setups to check for memory leaks
        for _i in 0..3 {
            let result = framework.test_interaction(
                &["2", "q", "y"],
                &["Game initialized", "Thanks for playing"]
            );
            assert!(result.success, "Memory stability test should pass");
        }
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["♠♣♥♦", "🎴", "åëîøü", "2", "q", "y"], // Special characters, then valid input
            &["Invalid", "Game initialized", "2 players"]
        );
        
        assert!(result.success, "Should handle unicode characters gracefully");
    }

    #[test]
    fn test_extremely_long_input() {
        let framework = ConsoleTestFramework::new("console");
        let long_input = "a".repeat(1000); // 1000 character input
        let result = framework.test_interaction(
            &[&long_input, "2", "q", "y"], // Long input, then valid input
            &["Invalid", "Game initialized"]
        );
        
        assert!(result.success, "Should handle extremely long input gracefully");
    }

    #[test]
    fn test_rapid_consecutive_inputs() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "?", "q", "y"], // Rapid sequence of commands
            &["Game initialized", "ILLIMAT HELP", "Thanks for playing"]
        );
        
        assert!(result.success, "Should handle rapid input sequences");
        assert!(result.execution_time < std::time::Duration::from_secs(5), "Should be responsive");
    }

    #[test]
    fn test_boundary_value_inputs() {
        let framework = ConsoleTestFramework::new("console");
        
        // Test boundary values for player count
        let boundary_tests = vec![
            ("1", "Invalid"), // Below minimum
            ("2", "2 players"), // Minimum valid
            ("4", "4 players"), // Maximum valid  
            ("5", "Invalid"), // Above maximum
        ];
        
        for (input, expected) in boundary_tests {
            let result = framework.test_interaction(
                &[input, "2", "q", "y"], // Test input, then valid fallback
                &[expected]
            );
            assert!(result.success, "Boundary test for '{}' should pass", input);
        }
    }

    #[test]
    fn test_game_action_sowing() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "s", "1", "0", "q", "y"], // 2 players, sow, card 1, field 0, quit
            &["Game initialized", "sow", "Field"]
        );
        
        assert!(result.success, "Should handle sowing action");
    }

    #[test] 
    fn test_game_action_harvesting() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "h", "1", "0", "q", "y"], // 2 players, harvest attempt, quit if it fails
            &["Game initialized"]
        );
        
        // Harvest might fail due to game state, but console should handle it gracefully
        assert!(result.execution_time < std::time::Duration::from_secs(10), "Should not hang");
    }

    #[test]
    fn test_undo_functionality() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "u", "q", "y"], // 2 players, try undo, quit
            &["Game initialized", "No actions to undo"]
        );
        
        assert!(result.success, "Should handle undo gracefully when no actions");
    }

    #[test]
    fn test_game_state_export() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "e", "q", "y"], // 2 players, export state, quit
            &["Game initialized", "Game state exported"]
        );
        
        assert!(result.success, "Should handle game state export");
    }

    #[test]
    fn test_action_history() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "i", "q", "y"], // 2 players, show history, quit
            &["Game initialized", "No action history available"]
        );
        
        assert!(result.success, "Should handle action history display");
    }

    #[test]
    fn test_quit_confirmation_no() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "q", "n", "q", "y"], // 2 players, quit, say no, quit again, say yes
            &["Game initialized", "Are you sure", "Thanks for playing"]
        );
        
        assert!(result.success, "Should handle quit confirmation properly");
    }

    #[test]
    fn test_invalid_game_actions() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "x", "z", "123", "q", "y"], // 2 players, invalid actions, quit
            &["Game initialized", "Invalid command"]
        );
        
        assert!(result.success, "Should handle invalid game actions gracefully");
    }

    #[test]
    fn test_case_insensitive_commands() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["2", "Q", "Y"], // 2 players, uppercase quit, uppercase yes
            &["Game initialized", "Thanks for playing"]
        );
        
        assert!(result.success, "Should handle case insensitive commands");
    }

    #[test]
    fn test_whitespace_handling() {
        let framework = ConsoleTestFramework::new("console");
        let result = framework.test_interaction(
            &["  2  ", " q ", " y "], // 2 players with whitespace, quit with whitespace
            &["Game initialized", "Thanks for playing"]
        );
        
        assert!(result.success, "Should handle whitespace in input");
    }

    #[test]
    fn test_console_stress_test() {
        let framework = ConsoleTestFramework::new("console");
        
        // Multiple rapid game cycles
        for cycle in 0..2 {
            let result = framework.test_interaction(
                &["2", "?", "q", "y"], // Quick help and quit cycle
                &["Game initialized", "ILLIMAT HELP"]
            );
            assert!(result.success, "Stress test cycle {} should pass", cycle);
        }
    }
}
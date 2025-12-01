//! Markdown parsing utilities for validation.
//!
//! This module provides utilities for parsing markdown sections
//! commonly found in spec files and change proposals.

/// Find a `## <header>` section and return its line number and content.
///
/// The content includes everything from the header line until the next
/// same-level header or end of file.
///
/// # Arguments
///
/// * `content` - The markdown content to search
/// * `header` - The header text to find (without the `## ` prefix)
///
/// # Returns
///
/// Returns `Some((line_number, section_content))` if found, `None` otherwise.
/// Line numbers are 1-indexed.
pub fn find_section(content: &str, header: &str) -> Option<(usize, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let target = format!("## {}", header);

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case(&target) || trimmed == target {
            // Found the header, collect content until next ## header
            let line_number = idx + 1; // 1-indexed
            let mut section_lines = vec![*line];

            for subsequent_line in lines.iter().skip(idx + 1) {
                let subsequent_trimmed = subsequent_line.trim();
                // Stop at next ## header (but not ### or deeper)
                if subsequent_trimmed.starts_with("## ")
                    && !subsequent_trimmed.starts_with("### ")
                {
                    break;
                }
                section_lines.push(*subsequent_line);
            }

            return Some((line_number, section_lines.join("\n")));
        }
    }

    None
}

/// Find all `### <prefix>` subsections within content.
///
/// # Arguments
///
/// * `content` - The markdown content to search
/// * `prefix` - The prefix to match (e.g., "Requirement:" matches "### Requirement: Login")
///
/// # Returns
///
/// Returns a vector of `(line_number, name, section_content)` tuples.
/// Line numbers are 1-indexed.
pub fn find_subsections(content: &str, prefix: &str) -> Vec<(usize, String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let target_prefix = format!("### {}", prefix);
    let mut results = Vec::new();

    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.starts_with(&target_prefix) {
            let line_number = idx + 1; // 1-indexed

            // Extract the name (everything after the prefix)
            let name = trimmed
                .strip_prefix("### ")
                .unwrap_or("")
                .strip_prefix(prefix)
                .unwrap_or("")
                .trim()
                .to_string();

            // Collect content until next ### header or ## header
            let mut section_lines = vec![line];
            idx += 1;

            while idx < lines.len() {
                let subsequent_line = lines[idx];
                let subsequent_trimmed = subsequent_line.trim();

                // Stop at next ### or ## header
                if subsequent_trimmed.starts_with("### ")
                    || (subsequent_trimmed.starts_with("## ")
                        && !subsequent_trimmed.starts_with("### "))
                {
                    break;
                }
                section_lines.push(subsequent_line);
                idx += 1;
            }

            results.push((line_number, name, section_lines.join("\n")));
        } else {
            idx += 1;
        }
    }

    results
}

/// Find all `#### Scenario:` blocks within content.
///
/// # Arguments
///
/// * `content` - The markdown content to search
///
/// # Returns
///
/// Returns a vector of `(line_number, scenario_name, scenario_content)` tuples.
/// Line numbers are 1-indexed.
pub fn extract_scenarios(content: &str) -> Vec<(usize, String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut results = Vec::new();

    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.starts_with("#### Scenario:") {
            let line_number = idx + 1; // 1-indexed

            // Extract the scenario name
            let name = trimmed
                .strip_prefix("#### Scenario:")
                .unwrap_or("")
                .trim()
                .to_string();

            // Collect content until next #### or ### or ## header
            let mut section_lines = vec![line];
            idx += 1;

            while idx < lines.len() {
                let subsequent_line = lines[idx];
                let subsequent_trimmed = subsequent_line.trim();

                // Stop at next header (##, ###, or ####)
                if subsequent_trimmed.starts_with("#### ")
                    || subsequent_trimmed.starts_with("### ")
                    || (subsequent_trimmed.starts_with("## ")
                        && !subsequent_trimmed.starts_with("### "))
                {
                    break;
                }
                section_lines.push(subsequent_line);
                idx += 1;
            }

            results.push((line_number, name, section_lines.join("\n")));
        } else {
            idx += 1;
        }
    }

    results
}

/// Check if content contains a WHEN clause.
pub fn has_when_clause(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        // Match patterns like "- **WHEN**", "**WHEN**", "WHEN "
        if trimmed.contains("**WHEN**") || trimmed.starts_with("WHEN ") {
            return true;
        }
    }
    false
}

/// Check if content contains a THEN clause.
pub fn has_then_clause(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        // Match patterns like "- **THEN**", "**THEN**", "THEN "
        if trimmed.contains("**THEN**") || trimmed.starts_with("THEN ") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== find_section tests ====================

    #[test]
    fn test_find_section_exists() {
        let content = r#"# Title

## Purpose

This is the purpose section.
It has multiple lines.

## Requirements

Some requirements here.
"#;

        let result = find_section(content, "Purpose");
        assert!(result.is_some());

        let (line, section_content) = result.unwrap();
        assert_eq!(line, 3); // "## Purpose" is on line 3
        assert!(section_content.contains("## Purpose"));
        assert!(section_content.contains("This is the purpose section."));
        assert!(section_content.contains("It has multiple lines."));
        assert!(!section_content.contains("## Requirements")); // Should stop before next section
    }

    #[test]
    fn test_find_section_not_exists() {
        let content = r#"# Title

## Purpose

This is the purpose section.
"#;

        let result = find_section(content, "NonExistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_section_case_insensitive() {
        let content = r#"## purpose
Some content here.
"#;

        let result = find_section(content, "Purpose");
        assert!(result.is_some());
    }

    #[test]
    fn test_find_section_with_subsections() {
        let content = r#"## Requirements

### Requirement: Login
User can login.

### Requirement: Logout
User can logout.

## Other Section
"#;

        let result = find_section(content, "Requirements");
        assert!(result.is_some());

        let (line, section_content) = result.unwrap();
        assert_eq!(line, 1);
        assert!(section_content.contains("### Requirement: Login"));
        assert!(section_content.contains("### Requirement: Logout"));
        assert!(!section_content.contains("## Other Section"));
    }

    #[test]
    fn test_find_section_at_end_of_file() {
        let content = r#"## First Section
Content 1.

## Last Section
Content 2.
No more sections after this.
"#;

        let result = find_section(content, "Last Section");
        assert!(result.is_some());

        let (line, section_content) = result.unwrap();
        assert_eq!(line, 4);
        assert!(section_content.contains("Content 2."));
        assert!(section_content.contains("No more sections after this."));
    }

    // ==================== find_subsections tests ====================

    #[test]
    fn test_find_subsections_multiple() {
        let content = r#"## Requirements

### Requirement: Login
User can login with credentials.

### Requirement: Logout
User can logout of the system.

### Requirement: Register
User can register a new account.
"#;

        let results = find_subsections(content, "Requirement:");
        assert_eq!(results.len(), 3);

        assert_eq!(results[0].0, 3); // Line number
        assert_eq!(results[0].1, "Login"); // Name
        assert!(results[0].2.contains("User can login"));

        assert_eq!(results[1].0, 6);
        assert_eq!(results[1].1, "Logout");
        assert!(results[1].2.contains("User can logout"));

        assert_eq!(results[2].0, 9);
        assert_eq!(results[2].1, "Register");
        assert!(results[2].2.contains("User can register"));
    }

    #[test]
    fn test_find_subsections_none() {
        let content = r#"## Requirements

Just some text without subsections.
"#;

        let results = find_subsections(content, "Requirement:");
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_subsections_with_scenarios() {
        let content = r#"### Requirement: Login
User can login.

#### Scenario: Valid credentials
- **WHEN** user provides valid credentials
- **THEN** user is logged in

#### Scenario: Invalid credentials
- **WHEN** user provides invalid credentials
- **THEN** error is shown
"#;

        let results = find_subsections(content, "Requirement:");
        assert_eq!(results.len(), 1);

        let (line, name, section_content) = &results[0];
        assert_eq!(*line, 1);
        assert_eq!(name, "Login");
        assert!(section_content.contains("#### Scenario: Valid credentials"));
        assert!(section_content.contains("#### Scenario: Invalid credentials"));
    }

    #[test]
    fn test_find_subsections_stops_at_next_section() {
        let content = r#"### Requirement: First
Content 1.

## New Section
This should not be included.

### Requirement: Second
Content 2.
"#;

        let results = find_subsections(content, "Requirement:");
        assert_eq!(results.len(), 2);

        assert!(!results[0].2.contains("## New Section"));
        assert!(!results[0].2.contains("This should not be included"));
    }

    // ==================== extract_scenarios tests ====================

    #[test]
    fn test_extract_scenarios_basic() {
        let content = r#"### Requirement: Login

#### Scenario: Valid login
- **WHEN** user enters valid credentials
- **THEN** user is logged in

#### Scenario: Invalid login
- **WHEN** user enters invalid credentials
- **THEN** error message is shown
"#;

        let results = extract_scenarios(content);
        assert_eq!(results.len(), 2);

        // Line 1: ### Requirement: Login
        // Line 2: (empty)
        // Line 3: #### Scenario: Valid login
        // Line 4: - **WHEN** ...
        // Line 5: - **THEN** ...
        // Line 6: (empty)
        // Line 7: #### Scenario: Invalid login
        assert_eq!(results[0].0, 3); // Line number
        assert_eq!(results[0].1, "Valid login"); // Name
        assert!(results[0].2.contains("**WHEN**"));
        assert!(results[0].2.contains("**THEN**"));

        assert_eq!(results[1].0, 7); // Line 7, not 8
        assert_eq!(results[1].1, "Invalid login");
    }

    #[test]
    fn test_extract_scenarios_with_when_then() {
        let content = r#"#### Scenario: User creates account
- **WHEN** user fills registration form
- **THEN** account is created
- **AND** confirmation email is sent
"#;

        let results = extract_scenarios(content);
        assert_eq!(results.len(), 1);

        let scenario = &results[0];
        assert!(has_when_clause(&scenario.2));
        assert!(has_then_clause(&scenario.2));
    }

    #[test]
    fn test_extract_scenarios_none() {
        let content = r#"### Requirement: Something
Just plain text without scenarios.
"#;

        let results = extract_scenarios(content);
        assert!(results.is_empty());
    }

    #[test]
    fn test_extract_scenarios_stops_at_next_requirement() {
        let content = r#"#### Scenario: First
Content 1.

### Requirement: Next
This should not be included.
"#;

        let results = extract_scenarios(content);
        assert_eq!(results.len(), 1);
        assert!(!results[0].2.contains("### Requirement"));
    }

    // ==================== has_when_clause tests ====================

    #[test]
    fn test_has_when_clause_bold() {
        let content = "- **WHEN** user clicks button";
        assert!(has_when_clause(content));
    }

    #[test]
    fn test_has_when_clause_plain() {
        let content = "WHEN user clicks button";
        assert!(has_when_clause(content));
    }

    #[test]
    fn test_has_when_clause_missing() {
        let content = "User clicks button";
        assert!(!has_when_clause(content));
    }

    #[test]
    fn test_has_when_clause_in_word() {
        // "when" as part of another word should not match
        let content = "Whenever user clicks";
        assert!(!has_when_clause(content));
    }

    // ==================== has_then_clause tests ====================

    #[test]
    fn test_has_then_clause_bold() {
        let content = "- **THEN** result is shown";
        assert!(has_then_clause(content));
    }

    #[test]
    fn test_has_then_clause_plain() {
        let content = "THEN result is shown";
        assert!(has_then_clause(content));
    }

    #[test]
    fn test_has_then_clause_missing() {
        let content = "Result is shown";
        assert!(!has_then_clause(content));
    }

    #[test]
    fn test_has_then_clause_in_word() {
        // "then" as part of another word should not match
        let content = "Authentication required";
        assert!(!has_then_clause(content));
    }

    // ==================== Integration tests ====================

    #[test]
    fn test_full_spec_parsing() {
        let content = r#"# Auth Spec

## Purpose

This spec defines authentication requirements.

## Requirements

### Requirement: User Login
The system SHALL allow users to login with email and password.

#### Scenario: Successful login
- **WHEN** user provides valid email and password
- **THEN** user is authenticated
- **AND** session token is returned

#### Scenario: Failed login
- **WHEN** user provides invalid credentials
- **THEN** authentication error is returned

### Requirement: User Logout
The system SHALL allow users to logout.

#### Scenario: Logout clears session
- **WHEN** authenticated user requests logout
- **THEN** session is invalidated
"#;

        // Find Purpose section
        let purpose = find_section(content, "Purpose");
        assert!(purpose.is_some());
        let (purpose_line, purpose_content) = purpose.unwrap();
        assert_eq!(purpose_line, 3);
        assert!(purpose_content.contains("authentication requirements"));

        // Find Requirements section
        let requirements = find_section(content, "Requirements");
        assert!(requirements.is_some());

        // Find all requirements
        let reqs = find_subsections(content, "Requirement:");
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].1, "User Login");
        assert_eq!(reqs[1].1, "User Logout");

        // Find scenarios in first requirement
        let login_scenarios = extract_scenarios(&reqs[0].2);
        assert_eq!(login_scenarios.len(), 2);
        assert_eq!(login_scenarios[0].1, "Successful login");
        assert_eq!(login_scenarios[1].1, "Failed login");

        // Verify WHEN/THEN clauses
        assert!(has_when_clause(&login_scenarios[0].2));
        assert!(has_then_clause(&login_scenarios[0].2));
    }
}

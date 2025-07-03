# Agents

This document contains information about AI agents and automated tools used in this project.

## Purpose

Document AI agent interactions, configurations, and best practices for maintaining and developing the Johansen Null Eigenspectra project.

## Agent Guidelines

### Code Generation

- Follow Rust best practices and conventions
- Maintain consistency with existing codebase style
- Ensure thread safety in parallel computations
- Write code comments in Chinese for internal documentation
- Use English for all user-facing messages, error messages, and CLI output
- Follow SOLID principles for clean and maintainable code design

### Testing

- Write comprehensive tests for new functionality
- Maintain test coverage for statistical computations
- Validate numerical accuracy of eigenvalue calculations
- Place tests in the `tests/` directory, not inline within `src/` files
- Follow the existing test structure with module-specific test directories

### Documentation

- Keep documentation up to date with code changes
- Provide clear examples for usage
- Document breaking changes and migration paths

### Security & Dependencies

- Keep dependencies up to date and audit for vulnerabilities
- Use minimal dependencies and prefer well-maintained crates
- Review dependency licenses for compatibility

### Error Handling

- Use appropriate Rust error handling patterns (Result, Option)
- Provide meaningful error messages in English for users
- Log detailed error information in Chinese for debugging
- Implement graceful error recovery where possible

### Code Quality

- Run `cargo fmt` before committing to ensure consistent formatting
- Use `cargo clippy` to catch common mistakes and improve code quality
- Ensure all code passes `cargo check` without warnings
- Write self-documenting code with clear variable and function names

### Project-Specific Requirements

- Understand the statistical nature of Johansen cointegration tests
- Maintain numerical precision in eigenvalue calculations
- Ensure reproducibility of simulation results using consistent random seeds
- Handle large-scale data efficiently for Monte Carlo simulations

## Development Workflow

### Git Workflow

- Use English names for all branches (no Chinese characters)
- Follow conventional branch naming patterns (e.g., `feature/`, `fix/`, `refactor/`)
- Use descriptive but concise branch names
- Before writing commit messages, execute the following commands in sequence:
  1. `git log --oneline -30` - Understand the overall commit message format and style patterns
  2. `git log -15` - Study detailed commit message writing techniques and conventions
  3. `git status` - Confirm the list of changed files and their status
  4. `git diff <file>` - Examine specific changes in each file to be committed
- Write commit messages based on the actual file differences and modifications discovered through git diff
- When multiple files are modified, group related changes by type (feat, fix, docs, style, refactor, test, chore) for atomic commits
- Always include scope in commit messages using format: `<type>[scope]: <description>`
- Use meaningful scopes that indicate the affected module or component
- For modifications involving multiple types, create a temporary branch first, complete all code changes, then merge back
- Only delete branches that were created by the AI agent, never delete user-created branches
- Follow the workflow: create temp branch → implement changes → commit by type → merge → cleanup AI-created branch only

### Code Review

- Verify statistical correctness of implementations
- Check performance implications of changes
- Do not enforce backward compatibility unless explicitly requested
- Focus on code quality and performance over legacy support

### Performance Considerations

- Profile computation-heavy operations
- Optimize memory usage for large-scale simulations
- Consider SIMD optimizations where applicable

## Notes

This file serves as a reference for AI agents working on this project to maintain consistency and quality standards.

# Contributing to paperx

Thank you for your interest in contributing to paperx! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- A LaTeX distribution (Tectonic, TeX Live, or MiKTeX)

### Setting Up the Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/paperx.git
   cd paperx
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/your-username/paperx.git
   ```
4. Build the project:
   ```bash
   cargo build
   ```
5. Run the tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Making Changes

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bugfix-name
   ```

2. Make your changes and test them thoroughly
3. Run the test suite:
   ```bash
   cargo test
   ```
4. Run clippy for code quality checks:
   ```bash
   cargo clippy -- -D warnings
   ```
5. Format your code:
   ```bash
   cargo fmt
   ```

### Testing Your Changes

Before submitting a pull request, please ensure:

- All tests pass (`cargo test`)
- The code compiles without warnings (`cargo build`)
- Clippy passes without warnings (`cargo clippy`)
- The code is properly formatted (`cargo fmt`)
- Your changes work with the example paper:
  ```bash
   # Create a test paper
   cargo run -- new test-paper --title "Test Paper" --author "Test Author"
   cd test-paper
   
   # Test building
   cargo run -- build
   
   # Test watching
   cargo run -- watch &
   # Make some changes to tex/main.tex
   # Verify it rebuilds automatically
   ```

### Commit Messages

Please follow these guidelines for commit messages:

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 50 characters
- Use the body to explain what and why, not how
- Reference issues and pull requests when applicable

Examples:
```
Add support for XeLaTeX engine

Implements XeLaTeX as a new engine option for building papers.
This addresses issue #123 by providing better Unicode support.
```

## Pull Request Process

1. Ensure your branch is up to date with the main branch:
   ```bash
   git checkout main
   git pull upstream main
   git checkout your-branch
   git rebase main
   ```

2. Push your changes to your fork:
   ```bash
   git push origin your-branch
   ```

3. Create a pull request on GitHub
4. Fill out the pull request template completely
5. Request review from maintainers

### Pull Request Template

When creating a pull request, please include:

- **Description**: What changes does this PR make?
- **Type of Change**: Bug fix, new feature, documentation, etc.
- **Testing**: How have you tested these changes?
- **Breaking Changes**: Does this PR introduce any breaking changes?
- **Checklist**: Confirm you've completed all necessary steps

## Areas for Contribution

We welcome contributions in the following areas:

### Code
- Bug fixes
- New features
- Performance improvements
- Code refactoring
- Test coverage improvements

### Documentation
- README improvements
- Code comments and documentation
- User guides and tutorials
- API documentation

### Templates
- New LaTeX templates for different paper types
- Template improvements and bug fixes
- Internationalization support

### Testing
- Unit tests
- Integration tests
- End-to-end tests
- Test data and fixtures

## Reporting Issues

When reporting issues, please include:

- **Description**: Clear description of the problem
- **Steps to Reproduce**: Detailed steps to reproduce the issue
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: OS, Rust version, LaTeX distribution
- **Additional Context**: Any other relevant information

## Feature Requests

When requesting features, please include:

- **Use Case**: Why is this feature needed?
- **Proposed Solution**: How should this feature work?
- **Alternatives**: What other solutions have you considered?
- **Additional Context**: Any other relevant information

## Release Process

Releases are managed by maintainers and follow semantic versioning:

- **Patch releases** (0.1.x): Bug fixes and minor improvements
- **Minor releases** (0.x.0): New features and enhancements
- **Major releases** (x.0.0): Breaking changes

## Questions?

If you have questions about contributing, please:

1. Check existing issues and discussions
2. Open a new issue with the "question" label
3. Contact maintainers directly

Thank you for contributing to paperx!

# Contributing to Mango Data Service

Thank you for your interest in contributing to Mango Data Service! This document provides guidelines for contributing to this project.

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+** ([Install Rust](https://rustup.rs/))
- **Git**
- **Optional**: PostgreSQL for testing database features

### Development Setup

1. **Fork and Clone**
   ```bash
       git clone https://github.com/coqui123/Pantera.git
    cd Pantera
   ```

2. **Environment Setup**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Install Dependencies and Build**
   ```bash
   cargo build
   cargo test
   ```

4. **Run Development Server**
   ```bash
   cargo run
   ```

## 🔧 Development Guidelines

### Code Style

We use the standard Rust formatting and linting tools:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run all checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Code Quality Standards

1. **Documentation**: All public APIs must have documentation comments
2. **Testing**: New features must include tests
3. **Error Handling**: Use proper error handling with `Result<T, E>`
4. **Performance**: Consider memory allocation and use `Cow<'_, str>` where appropriate
5. **Security**: Validate all inputs and use parameterized database queries

### Commit Message Format

Use conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `perf`: Performance improvements
- `chore`: Build process or auxiliary tool changes

**Examples:**
```
feat(api): add bulk historical data endpoint
fix(database): resolve connection pool exhaustion
docs(readme): update installation instructions
perf(cache): implement zero-copy string operations
```

## 🧪 Testing

### Running Tests

```bash
# Run unit tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_rate_limiting

# Run integration tests
cargo test --test integration
```

### Test Coverage

- Unit tests for core functionality
- Integration tests for API endpoints
- Performance tests for critical paths
- Error case testing

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_symbol_validation() {
        let service = create_test_service().await;
        let result = service.validate_symbol("AAPL").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
```

## 📝 Documentation

### API Documentation

- All public functions must have doc comments
- Include examples in doc comments where appropriate
- Update API_REFERENCE.md for new endpoints

### Code Documentation

```rust
/// Fetches historical data for a symbol with caching
/// 
/// # Arguments
/// 
/// * `symbol` - The stock symbol (e.g., "AAPL")
/// * `interval` - The time interval ("1d", "1h", etc.)
/// * `force_refresh` - Whether to bypass cache
/// 
/// # Returns
/// 
/// A vector of historical prices sorted by timestamp descending
/// 
/// # Example
/// 
/// ```rust
/// let data = service.fetch_historical_data("AAPL", "1d", false).await?;
/// ```
pub async fn fetch_historical_data(
    &self,
    symbol: &str,
    interval: &str,
    force_refresh: bool,
) -> Result<Vec<HistoricalPrice>> {
    // Implementation
}
```

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment**: OS, Rust version, dependency versions
2. **Steps to Reproduce**: Clear, minimal steps
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Logs**: Relevant error messages or logs
6. **Minimal Example**: Code that reproduces the issue

**Template:**
```markdown
## Bug Description
Brief description of the issue

## Environment
- OS: Windows 10/Linux/macOS
- Rust Version: 1.70+
- Mango Data Service Version: 0.1.0

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Error Logs
```
Paste error logs here
```

## Additional Context
Any other relevant information
```

## ✨ Feature Requests

For new features, please:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case** - why is this needed?
3. **Propose the solution** - how should it work?
4. **Consider alternatives** - are there other approaches?

## 🔄 Pull Request Process

### Before Submitting

1. **Create an issue** to discuss major changes
2. **Fork the repository** and create a feature branch
3. **Write tests** for new functionality
4. **Update documentation** as needed
5. **Run all checks** locally

### Pull Request Guidelines

1. **Branch Naming**: `feature/description` or `fix/description`
2. **Clear Title**: Descriptive title following conventional commits
3. **Description**: Explain what and why, not just how
4. **Link Issues**: Reference related issues
5. **Small PRs**: Keep changes focused and reviewable

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] New tests added for new functionality
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or clearly documented)
```

### Review Process

1. **Automated Checks**: All CI checks must pass
2. **Code Review**: At least one approval required
3. **Testing**: Verify functionality works as expected
4. **Documentation**: Ensure docs are updated
5. **Merge**: Squash and merge with clean commit message

## 🏗️ Architecture Guidelines

### Performance Considerations

- Use `Cow<'_, str>` for string fields that might be borrowed
- Implement concurrent caching with `DashMap`
- Use connection pooling for database operations
- Consider rate limiting for external API calls

### Security Best Practices

- Validate all user inputs
- Use parameterized database queries
- Implement proper rate limiting
- Log security-relevant events
- Follow principle of least privilege

### Error Handling

```rust
// Use custom error types
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid symbol: {symbol}")]
    InvalidSymbol { symbol: String },
}
```

## 🚀 Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Create release PR
- [ ] Tag release after merge
- [ ] Publish to crates.io (if applicable)

## 📞 Getting Help

- **Issues**: [GitHub Issues](https://github.com/coqui123/Pantera/issues)
- **Discussions**: [GitHub Discussions](https://github.com/coqui123/Pantera/discussions)
- **Documentation**: Check README.md and docs/ directory

## 📄 Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## 🙏 Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes for significant contributions
- GitHub contributors page

Thank you for contributing to Mango Data Service! 🚀 
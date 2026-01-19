# GitHub Actions Workflows

This directory contains the CI/CD pipeline configuration for LLM-Forge.

## 📋 Workflows Overview

| Workflow | Status | Trigger | Purpose |
|----------|--------|---------|---------|
| [PR Validation](.github/workflows/pr-validation.yml) | - | Pull Requests | Code quality checks before merge |
| [CI Pipeline](.github/workflows/ci.yml) | - | Push to main/develop | Continuous integration |
| [Release](.github/workflows/release.yml) | - | Version tags | Automated releases |
| [Security Scan](.github/workflows/security.yml) | - | Push, PR, Daily | Security analysis |
| [Performance](.github/workflows/performance.yml) | - | Push, PR, Weekly | Performance monitoring |
| [Dependabot Auto-Merge](.github/workflows/dependabot-auto-merge.yml) | - | Dependabot PRs | Auto-merge dependencies |
| [Stale Management](.github/workflows/stale.yml) | - | Daily | Clean up inactive issues/PRs |

## 🚀 Quick Start

### For Contributors

1. **Create a branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes and commit**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

3. **Push and create PR**
   ```bash
   git push origin feature/my-feature
   ```

4. **Wait for checks to pass**
   - All PR validation checks must pass
   - Code coverage must be maintained
   - No security vulnerabilities

### For Maintainers

1. **Creating a Release**
   ```bash
   # Update version in package.json
   npm version 1.2.3 --no-git-tag-version

   # Commit and tag
   git add package.json package-lock.json
   git commit -m "chore: bump version to 1.2.3"
   git tag v1.2.3
   git push origin main --tags
   ```

2. **Manual Workflow Dispatch**
   - Go to Actions tab
   - Select workflow
   - Click "Run workflow"
   - Fill in required inputs

## 📊 Workflow Details

### Pull Request Validation

**What it does:**
- ✅ Type checking
- ✅ Linting
- ✅ Format checking
- ✅ Tests on Node 20 & 21
- ✅ Coverage verification (92%+ target)
- ✅ Build verification

**When it runs:**
- When PR is opened
- When PR is updated
- When PR is ready for review

**Notes:**
- Draft PRs are skipped
- Results posted as PR comment

### Continuous Integration

**What it does:**
- ✅ Quality checks
- ✅ Multi-OS testing (Ubuntu, macOS, Windows)
- ✅ Performance benchmarks
- ✅ Build verification
- ✅ Package installation test
- ✅ Coverage reporting
- ✅ Documentation deployment

**When it runs:**
- On push to main or develop
- Can be manually triggered

### Release Pipeline

**What it does:**
- ✅ Version validation
- ✅ Full test suite
- ✅ Build package
- ✅ Publish to npm
- ✅ Publish to GitHub Packages
- ✅ Create GitHub release
- ✅ Generate changelog

**When it runs:**
- When version tag is pushed (e.g., v1.2.3)
- Can be manually triggered

**Required Secrets:**
- `NPM_TOKEN` - npm authentication

### Security Scanning

**What it does:**
- ✅ Dependency vulnerability scan
- ✅ CodeQL static analysis
- ✅ Secret detection
- ✅ License compliance
- ✅ SAST analysis
- ✅ OSSF scorecard

**When it runs:**
- On push to main/develop
- On pull requests
- Daily at 2 AM UTC
- Can be manually triggered

### Performance Monitoring

**What it does:**
- ✅ Run benchmarks
- ✅ Compare with baseline
- ✅ Track performance over time
- ✅ Detect regressions

**When it runs:**
- On push to main
- On pull requests
- Weekly on Sundays at 3 AM UTC
- Can be manually triggered

## 🔐 Security

### Secrets Required

Configure these in `Settings > Secrets and variables > Actions`:

| Secret | Description | Required For |
|--------|-------------|--------------|
| `NPM_TOKEN` | npm registry token | Releases |
| `CODECOV_TOKEN` | Codecov upload token | Coverage reporting |

### Security Best Practices

- ✅ All secrets stored in GitHub Secrets
- ✅ No hardcoded credentials
- ✅ Automated security scanning
- ✅ Dependency updates via Dependabot
- ✅ License compliance checking

## 📈 Monitoring

### Viewing Workflow Runs

1. Go to the **Actions** tab
2. Select a workflow from the left sidebar
3. View run history and logs

### Debugging Failed Workflows

1. Click on the failed run
2. Expand the failed job
3. Review the logs
4. Check for error messages

### Common Issues

**Tests fail in CI but pass locally:**
- Ensure you're using the same Node version
- Run `npm ci` instead of `npm install`
- Check for environment-specific issues

**Coverage below threshold:**
- Add tests for new code
- Run `npm run test:coverage` locally

**Build failures:**
- Check TypeScript errors
- Ensure all dependencies are installed
- Verify build configuration

## 🔄 Workflow Updates

### Updating Workflows

1. Edit workflow files in `.github/workflows/`
2. Test on a feature branch first
3. Create PR with changes
4. Merge after review

### Testing Workflows

```bash
# Install act (GitHub Actions local runner)
# https://github.com/nektos/act

# Test a workflow locally
act pull_request
```

## 📚 Documentation

- [Full CI/CD Documentation](../../docs/CI_CD_PIPELINE.md)
- [GitHub Actions Docs](https://docs.github.com/actions)
- [Workflow Syntax Reference](https://docs.github.com/actions/reference/workflow-syntax-for-github-actions)

## 🤝 Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines on contributing to this project.

## 📝 License

This project is licensed under the Apache-2.0 License - see the [LICENSE](../../LICENSE) file for details.

---

**Status:** ✅ All workflows operational
**Last Updated:** November 8, 2025

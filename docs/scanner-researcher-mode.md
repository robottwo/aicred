# 🔍 Scanner Researcher Mode

The Scanner Researcher mode is a specialized Roo Code mode designed to research AI applications and create comprehensive scanner specifications for the GenAI Keyfinder project.

## Overview

This mode automates the process of:
1. **Researching** AI applications from GitHub repositories or documentation
2. **Analyzing** configuration patterns and file formats
3. **Discovering** where applications store API keys and configuration files
4. **Creating** detailed scanner specifications with implementation guidance
5. **Providing** test cases and examples for unit testing

## When to Use

Use the Scanner Researcher mode when you need to:
- Add support for a new AI application to GenAI Keyfinder
- Research an application's configuration patterns
- Create scanner implementation specifications
- Generate test cases for scanner validation
- Document configuration file locations across platforms

## How It Works

### 1. Research Phase
The mode starts by examining the provided GitHub repository or application link:
- Analyzes the source code structure
- Identifies configuration-related files
- Looks for documentation about setup and configuration
- Searches for examples and sample configurations

### 2. Analysis Phase
Next, it analyzes the discovered information:
- Configuration file formats (JSON, YAML, TOML, etc.)
- File naming patterns and locations
- API key storage mechanisms
- Environment variable usage
- Cross-platform differences

### 3. Specification Creation
Finally, it creates a comprehensive specification document:
- Scanner implementation guidance
- File detection logic
- Key extraction patterns
- Test configurations
- Platform-specific considerations

## Usage Examples

### Example 1: Research a VSCode Extension
```
Research the Continue.dev VSCode extension and create a scanner specification.
GitHub: https://github.com/continuedev/continue
```

### Example 2: Research a CLI Tool
```
Research the Ollama CLI tool and create a scanner specification.
GitHub: https://github.com/ollama/ollama
```

### Example 3: Research a Web Application
```
Research the Open WebUI application and create a scanner specification.
GitHub: https://github.com/open-webui/open-webui
```

## Output Format

The mode generates a comprehensive specification document with:

### Application Information
- Name, description, and repository URL
- Primary language and license
- Application type (VSCode extension, CLI, web app, etc.)

### Configuration Analysis
- Supported file formats (JSON, YAML, etc.)
- Configuration file locations for each platform
- Environment variable patterns
- Key configuration files

### API Key Patterns
- Supported AI providers
- Key validation rules
- Configuration examples
- Environment variable examples

### Implementation Guidance
- File detection logic
- Key extraction functions
- Scanner name and supported providers
- Platform-specific considerations

### Test Cases
- Valid configuration examples
- Invalid configuration examples
- Expected results for each test case

## Integration with GenAI Keyfinder

The specifications created by this mode are designed to be directly implementable in the GenAI Keyfinder project. Each specification includes:

1. **Rust Implementation Code**: Ready-to-use scanner implementation
2. **Test Configurations**: JSON/YAML examples for unit tests
3. **File Paths**: Platform-specific configuration locations
4. **Provider Support**: List of AI providers the scanner can detect

## Best Practices

### Research Tips
- Start with the application's documentation
- Look for setup and configuration guides
- Check for example configurations
- Review GitHub issues for configuration problems
- Examine the source code for hardcoded paths

### Analysis Tips
- Note platform differences in file locations
- Identify multiple configuration methods
- Look for deprecated configuration options
- Consider security implications
- Check for encrypted vs plain text storage

### Specification Tips
- Provide complete file detection logic
- Include confidence scoring for keys
- Add comprehensive test cases
- Document platform-specific behavior
- Consider edge cases and error conditions

## Example Specifications

See the `examples/` directory for real scanner specifications created with this mode:

- [`continue-dev-scanner-spec.md`](examples/continue-dev-scanner-spec.md) - Continue.dev VSCode extension
- [More examples to be added]

## Contributing

To improve the Scanner Researcher mode:
1. Test it with various AI applications
2. Provide feedback on the research process
3. Suggest new analysis techniques
4. Share successful scanner implementations
5. Report issues with specifications

## References

- [GenAI Keyfinder Project](https://github.com/your-org/genai-keyfinder)
- [Scanner Plugin Interface](core/src/scanners/mod.rs)
- [Example Scanner Implementations](core/src/scanners/)

## Troubleshooting

### Common Issues

**Q: The mode can't find configuration information**
A: Check if the repository has documentation in a different location (wiki, docs folder, etc.)

**Q: The specification seems incomplete**
A: Some applications have multiple configuration methods - research all of them

**Q: Platform-specific paths are missing**
A: Check the application's source code for platform detection logic

**Q: Test cases are failing**
A: Verify the key validation rules and adjust the confidence scoring

### Getting Help
- Check existing scanner implementations for patterns
- Review the scanner plugin interface documentation
- Test with simple configurations first
- Use the template as a starting point
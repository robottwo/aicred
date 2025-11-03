/*
Package aicred provides Go bindings for the aicred library.

This package allows Go applications to scan for GenAI API keys and configurations
across various providers and applications.

Basic Usage:

	import (
		"fmt"
		"log"

		aicred "github.com/robottwo/aicred/bindings/go/genai_keyfinder"
	)

	func main() {
		// Scan with default options
		result, err := aicred.Scan(aicred.ScanOptions{})
		if err != nil {
			log.Fatal(err)
		}

		fmt.Printf("Found %d keys\n", len(result.Keys))
		for _, key := range result.Keys {
			fmt.Printf("%s: %s\n", key.Provider, key.Redacted)
		}
	}

Security:

By default, all secrets are redacted. Only use IncludeFullValues: true in secure environments.

Supported Providers:

  - OpenAI
  - Anthropic (Claude)
  - Hugging Face
  - Ollama
  - Groq
  - LiteLLM

Supported Applications:

  - Roo Code
  - Claude Desktop
  - Ragit
  - LangChain applications
*/
package aicred

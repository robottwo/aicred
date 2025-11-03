#ifndef GENAI_KEYFINDER_H
#define GENAI_KEYFINDER_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Scan for GenAI credentials and configurations
 *
 * # Parameters
 * - `home_path`: UTF-8 encoded home directory path (null-terminated C string)
 * - `options_json`: UTF-8 encoded JSON options (null-terminated C string)
 *
 * # Returns
 * UTF-8 encoded JSON string containing scan results. Caller must free with [`aicred_free`].
 * Returns NULL on error.
 *
 * # Example options_json:
 * ```json
 * {
 *   "include_full_values": false,
 *   "max_file_size": 1048576,
 *   "only_providers": ["openai", "anthropic"],
 *   "exclude_providers": []
 * }
 * ```
 *
 * # Safety
 *
 * Both pointers must be either null or point to valid null-terminated C strings.
 */
char *aicred_scan(const char *home_path, const char *options_json);

/**
 * Free a string returned by aicred_scan
 *
 * # Safety
 *
 * The pointer must be either null or point to a string allocated by this library.
 */
void aicred_free(char *ptr);

/**
 * Get library version string
 *
 * Returns a static version string that does not need to be freed.
 */
const char *aicred_version(void);

/**
 * Get last error message (thread-local)
 *
 * Returns a pointer to the last error message, or null if no error occurred.
 * The returned pointer is valid until the next call to any aicred function.
 */
const char *aicred_last_error(void);

/**
 * Get list of available provider plugins
 *
 * Returns a JSON array of provider names as a UTF-8 encoded string.
 * Caller must free the returned string with [`aicred_free`].
 * Returns NULL on error.
 *
 * # Example return value:
 * ```json
 * ["openai", "anthropic", "huggingface", "groq", "ollama", "litellm", "common-config"]
 * ```
 *
 * # Safety
 *
 * The returned pointer must be freed by the caller using [`aicred_free`].
 */
char *aicred_list_providers(void);

/**
 * Get list of available scanner plugins
 *
 * Returns a JSON array of scanner names as a UTF-8 encoded string.
 * Caller must free the returned string with [`aicred_free`].
 * Returns NULL on error.
 *
 * # Example return value:
 * ```json
 * ["ragit", "claude-desktop", "roo-code", "langchain", "gsh"]
 * ```
 *
 * # Safety
 *
 * The returned pointer must be freed by the caller using [`aicred_free`].
 */
char *aicred_list_scanners(void);

#endif /* GENAI_KEYFINDER_H */
